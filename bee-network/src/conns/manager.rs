// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use crate::{
    interaction::events::InternalEventSender,
    peers::{BannedAddrList, BannedPeerList, PeerList},
    transport::build_transport,
    PEER_LIMIT,
};

use super::{
    connection::{MuxedConnection, Origin},
    errors::Error,
    spawn_connection_handler,
};

use bee_common::{shutdown::ShutdownListener, worker::Error as WorkerError};

use futures::{prelude::*, select};
use libp2p::{
    core::{muxing::StreamMuxerBox, transport::ListenerEvent},
    identity,
    multiaddr::Protocol,
    Multiaddr, PeerId, Transport,
};
use log::*;

use std::{
    io,
    net::IpAddr,
    pin::Pin,
    sync::atomic::{AtomicUsize, Ordering},
};

type ListenerUpgrade = Pin<Box<(dyn Future<Output = Result<(PeerId, StreamMuxerBox), io::Error>> + Send + 'static)>>;
type PeerListener = Pin<Box<dyn Stream<Item = Result<ListenerEvent<ListenerUpgrade, io::Error>, io::Error>> + Send>>;

pub static NUM_LISTENER_EVENT_PROCESSING_ERRORS: AtomicUsize = AtomicUsize::new(0);

pub struct ConnectionManager {
    pub listen_address: Multiaddr,
    internal_event_sender: InternalEventSender,
    peers: PeerList,
    banned_addrs: BannedAddrList,
    banned_peers: BannedPeerList,
    peer_listener: PeerListener,
    shutdown_listener: ShutdownListener,
}

impl ConnectionManager {
    pub fn new(
        local_keys: identity::Keypair,
        bind_address: Multiaddr,
        internal_event_sender: InternalEventSender,
        shutdown_listener: ShutdownListener,
        peers: PeerList,
        banned_addrs: BannedAddrList,
        banned_peers: BannedPeerList,
    ) -> Result<Self, Error> {
        // Create underlying Tcp connection and negotiate Noise and Mplex/Yamux
        let transport = build_transport(&local_keys).map_err(|_| Error::CreatingTransportFailed)?;

        let mut peer_listener = transport
            .listen_on(bind_address.clone())
            .map_err(|_| Error::BindingAddressFailed(bind_address))?;

        let listen_address =
            if let Some(Some(Ok(ListenerEvent::NewAddress(listen_address)))) = peer_listener.next().now_or_never() {
                trace!("listening address = {}", listen_address);
                listen_address
            } else {
                return Err(Error::NotListeningError);
            };

        trace!("Accepting connections on {}.", listen_address);

        Ok(Self {
            listen_address,
            internal_event_sender,
            peers,
            banned_peers,
            banned_addrs,
            peer_listener,
            shutdown_listener,
        })
    }

    pub async fn run(self) -> Result<(), WorkerError> {
        trace!("Connection Manager started.");

        let ConnectionManager {
            internal_event_sender,
            peers,
            banned_peers,
            banned_addrs,
            peer_listener,
            shutdown_listener,
            ..
        } = self;

        let mut fused_incoming_streams = peer_listener.fuse();
        let mut fused_shutdown_listener = shutdown_listener.fuse();

        loop {
            select! {
                _ = fused_shutdown_listener => {
                    trace!("Connection Manager received shutdown signal.");
                    break;
                },
                listener_event = fused_incoming_streams.next() => {
                    if let Some(listener_event) = listener_event {
                        if let Ok(listener_event) = listener_event {
                            if let Some((upgrade, peer_address)) = listener_event.into_upgrade() {

                                // process_listener_event(
                                //     listener_event,
                                //     upgrade,
                                //     peer_address,
                                //     &peers,
                                //     &banned_peers,
                                //     &banned_addrs,
                                //     &internal_event_sender,
                                // )
                                // .await//.expect("process_listener_event")

                                // FIXME: unwrap (but is blowup even possible?)
                                let ip_address = match peer_address.iter().next().unwrap() {
                                    Protocol::Ip4(ip_addr) => IpAddr::V4(ip_addr),
                                    Protocol::Ip6(ip_addr) => IpAddr::V6(ip_addr),
                                    _ => {
                                        warn!("Invalid multiaddress.");
                                        NUM_LISTENER_EVENT_PROCESSING_ERRORS.fetch_add(1, Ordering::Relaxed);
                                        continue;
                                    }
                                };

                                if banned_addrs.contains(&ip_address) {
                                    warn!("Ignoring peer. Cause: '{}' is banned.", ip_address);
                                    NUM_LISTENER_EVENT_PROCESSING_ERRORS.fetch_add(1, Ordering::Relaxed);
                                    continue;
                                }

                                let (peer_id, muxer) = match upgrade.await {
                                    Ok(u) => u,
                                    Err(_) => {
                                        NUM_LISTENER_EVENT_PROCESSING_ERRORS.fetch_add(1, Ordering::Relaxed);
                                        continue;
                                    }
                                };

                                if banned_peers.contains(&peer_id) {
                                    warn!("Ignoring peer. Cause: '{}' is banned.", peer_id);
                                    NUM_LISTENER_EVENT_PROCESSING_ERRORS.fetch_add(1, Ordering::Relaxed);
                                    continue;
                                }

                                if peers.num_connected() >= PEER_LIMIT.load(Ordering::Relaxed) {
                                    warn!(
                                        "Ignoring peer. Cause: Peer limit ({}) reached.",
                                        PEER_LIMIT.load(Ordering::Relaxed)
                                    );
                                    NUM_LISTENER_EVENT_PROCESSING_ERRORS.fetch_add(1, Ordering::Relaxed);
                                    continue;
                                }

                                if peers.contains_peer(&peer_id) {
                                    trace!("Already connected to {}", peer_id);
                                    NUM_LISTENER_EVENT_PROCESSING_ERRORS.fetch_add(1, Ordering::Relaxed);
                                    continue;
                                }

                                let connection = MuxedConnection::new(peer_id, peer_address, muxer, Origin::Inbound);

                                trace!(
                                    "Successfully established inbound connection to {} ({}).",
                                    connection.peer_address,
                                    connection.peer_id,
                                );

                                if let Err(e) = spawn_connection_handler(connection, internal_event_sender.clone())
                                .await
                                {
                                    error!("Error spawning connection handler. Error: {}", e);
                                    NUM_LISTENER_EVENT_PROCESSING_ERRORS.fetch_add(1, Ordering::Relaxed);
                                    continue;
                                }
                            }
                        } else {
                            error!("Listener event stream failure.");
                            NUM_LISTENER_EVENT_PROCESSING_ERRORS.fetch_add(1, Ordering::Relaxed);
                            continue;
                        }
                    } else {
                        error!("Fatal: Listener event stream stopped.");
                        NUM_LISTENER_EVENT_PROCESSING_ERRORS.fetch_add(1, Ordering::Relaxed);
                        break;
                    }
                },
            }
        }

        trace!("Connection Manager stopped.");
        Ok(())
    }
}

#[inline]
async fn process_listener_event(// listener_event: ListenerEvent<ListenerUpgrade, io::Error>,
    // listener_upgrade: ListenerUpgrade,
    // peer_address: Multiaddr,
    // peers: &PeerList,
    // banned_peers: &BannedPeerList,
    // banned_addrs: &BannedAddrList,
    // internal_event_sender: &InternalEventSender,
) {
    // // Upgrade TokioTcpConfig
    // if let Some((upgrade, peer_address)) = listener_event.into_upgrade() {
    //     let ip_address = match peer_address.iter().next().unwrap() {
    //         Protocol::Ip4(ip_addr) => IpAddr::V4(ip_addr),
    //         Protocol::Ip6(ip_addr) => IpAddr::V6(ip_addr),
    //         _ => unreachable!("wrong multiaddr"),
    //     };

    //     if banned_addrs.contains(&ip_address) {
    //         warn!("Ignoring peer. Cause: '{}' is banned.", ip_address);
    //         return; // Ok(());
    //     }

    //     // TODO: error handling
    //     let (peer_id, muxer) = upgrade.await.expect("upgrade failed");

    //     if banned_peers.contains(&peer_id) {
    //         warn!("Ignoring peer. Cause: '{}' is banned.", peer_id);
    //         return; // Ok(());
    //     }

    //     if peers.num_connected() >= PEER_LIMIT.load(Ordering::Relaxed) {
    //         warn!(
    //             "Ignoring peer. Cause: Peer limit ({}) reached.",
    //             PEER_LIMIT.load(Ordering::Relaxed)
    //         );
    //         return; // Ok(());
    //     }

    //     if peers.contains_peer(&peer_id) {
    //         trace!("Already connected to {}", peer_id);
    //         return; // Ok(());
    //     }

    //     let connection = MuxedConnection::new(peer_id, peer_address, muxer, Origin::Inbound);

    //     trace!(
    //         "Successfully established inbound connection to {} ({}).",
    //         connection.peer_address,
    //         connection.peer_id,
    //     );

    //     // let internal_event_sender = internal_event_sender.clone();

    //     // FIXME: map error
    //     if let Err(_) = spawn_connection_handler(connection, internal_event_sender.clone()).await {
    //         todo!("spawn_connection_handler error handling")
    //     // Err(WorkerError(Box::new(io::Error::new(
    //     //     io::ErrorKind::InvalidData,
    //     //     "spawn_connection_handler",
    //     // ))))
    //     } else {
    //         // Ok(())
    //     }
    // } else {
    //     // TODO: handle other listener events
    //     trace!("Not an upgrade event.");
    //     todo!("spawn_connection_handler error handling");
    //     // Err(WorkerError(Box::new(io::Error::new(
    //     //     io::ErrorKind::InvalidData,
    //     //     "not an upgrade event",
    //     // ))))
    // }
}
