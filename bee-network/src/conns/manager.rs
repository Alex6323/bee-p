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
    peers::{BannedAddrList, BannedPeerList, PeerInfo, PeerList, PeerRelation, PeerState},
    transport::build_transport,
    Multiaddr, PeerId, ShortId,
};

use super::{errors::Error, spawn_connection_handler, Origin};

use bee_common::{shutdown::ShutdownListener, worker::Error as WorkerError};

use futures::{prelude::*, select};
use libp2p::{
    core::{muxing::StreamMuxerBox, transport::ListenerEvent},
    identity, Transport,
};
use log::*;

use std::{
    io,
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

                                // TODO: try again to move this block into its own function (beware: lifetime issues ahead!!!)

                                // Prevent accepting from banned addresses.
                                let peer_address_str = peer_address.to_string();
                                if banned_addrs.contains(&peer_address_str) {
                                    trace!("Ignoring peer. Cause: '{}' is banned.", peer_address_str);
                                    NUM_LISTENER_EVENT_PROCESSING_ERRORS.fetch_add(1, Ordering::Relaxed);
                                    continue;
                                }

                                let (peer_id, muxer) = match upgrade.await {
                                    Ok(u) => u,
                                    Err(_) => {
                                        trace!("Ignoring peer. Cause: Handshake failed.");
                                        NUM_LISTENER_EVENT_PROCESSING_ERRORS.fetch_add(1, Ordering::Relaxed);
                                        continue;
                                    }
                                };

                                // Prevent accepting duplicate connections.
                                if let Ok(connected) = peers.is(&peer_id, |_, state| state.is_connected()) {
                                    if connected {
                                        trace!("Already connected to {}", peer_id);
                                        NUM_LISTENER_EVENT_PROCESSING_ERRORS.fetch_add(1, Ordering::Relaxed);
                                        continue;
                                    }
                                }

                                // Prevent accepting banned peers.
                                if banned_peers.contains(&peer_id) {
                                    trace!("Ignoring peer. Cause: '{}' is banned.", peer_id);
                                    NUM_LISTENER_EVENT_PROCESSING_ERRORS.fetch_add(1, Ordering::Relaxed);
                                    continue;
                                }

                                let peer_info = if let Ok(peer_info) = peers.get_info(&peer_id) {
                                    // If we have this peer id in our peerlist (but are not already connected to it),
                                    // then we allow the connection.
                                    peer_info
                                } else {
                                    let peer_info = PeerInfo {
                                        address: peer_address,
                                        alias: None,
                                        relation: PeerRelation::Unknown
                                    };

                                    if peers.insert(peer_id.clone(), peer_info.clone(), PeerState::Disconnected).is_err() {
                                        trace!("Ignoring peer. Cause: Denied by peerlist.");
                                        NUM_LISTENER_EVENT_PROCESSING_ERRORS.fetch_add(1, Ordering::Relaxed);
                                        continue;
                                    } else {
                                        // We also allow for a certain number of unknown peers.
                                        info!("Allowing connection to unknown peer '{}' [{}]", peer_id.short(), peer_info.address);

                                        peer_info
                                    }
                                };

                                log_inbound_connection_success(&peer_id, &peer_info);

                                if let Err(e) = spawn_connection_handler(peer_id, peer_info, muxer, Origin::Inbound, internal_event_sender.clone())
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
fn log_inbound_connection_success(peer_id: &PeerId, peer_info: &PeerInfo) {
    if let Some(alias) = peer_info.alias.as_ref() {
        info!(
            "Established (inbound) connection with '{}/{}' [{}].",
            alias,
            peer_id.short(),
            peer_info.address,
        )
    } else {
        info!(
            "Established (inbound) connection with '{}' [{}].",
            peer_id.short(),
            peer_info.address,
        );
    }
}
