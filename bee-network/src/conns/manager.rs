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

use crate::{interaction::events::EventSender, peers::KnownPeerList, transport::build_transport};

use super::{
    connection::{Connection, Origin},
    spawn_reader_writer,
};

use bee_common::{shutdown::ShutdownListener, worker::Error as WorkerError};

use futures::{prelude::*, select};
use libp2p::{
    core::{muxing::StreamMuxerBox, transport::ListenerEvent},
    identity, Multiaddr, PeerId, Transport,
};
use log::*;

use std::{io, pin::Pin};

type ListenerUpgrade = Pin<Box<(dyn Future<Output = Result<(PeerId, StreamMuxerBox), io::Error>> + Send + 'static)>>;
type Listener = Pin<Box<dyn Stream<Item = Result<ListenerEvent<ListenerUpgrade, io::Error>, io::Error>> + Send>>;

pub struct ConnectionManager {
    #[allow(dead_code)]
    listener_address: Multiaddr,
    internal_event_sender: EventSender,
    known_peers: KnownPeerList,
    listener: Listener,
    shutdown_listener: ShutdownListener,
}

impl ConnectionManager {
    pub fn new(
        local_keys: identity::Keypair,
        bind_address: Multiaddr,
        internal_event_sender: EventSender,
        shutdown_listener: ShutdownListener,
        known_peers: KnownPeerList,
    ) -> Self {
        trace!("Starting Connection Manager...");

        let transport = build_transport(&local_keys).expect("error building transport");

        let mut listener = transport.listen_on(bind_address).expect("Error binding Peer Listener.");

        let listener_address =
            if let Some(Some(Ok(ListenerEvent::NewAddress(address)))) = listener.next().now_or_never() {
                trace!("listening address = {}", address);
                address
            } else {
                panic!("Not listening on an address!");
            };

        trace!("Accepting connections on {}.", listener_address);

        Self {
            listener_address,
            internal_event_sender,
            known_peers,
            listener,
            shutdown_listener,
        }
    }

    pub async fn run(self) -> Result<(), WorkerError> {
        trace!("Connection Manager running...");

        let ConnectionManager {
            internal_event_sender,
            known_peers,
            listener,
            shutdown_listener,
            ..
        } = self;

        let mut fused_incoming_streams = listener.fuse();
        let mut fused_shutdown_listener = shutdown_listener.fuse();

        loop {
            select! {
                _ = fused_shutdown_listener => {
                    trace!("Breaking Connection Manager. Cause: shutdown listener");
                    break;
                },
                listener_event = fused_incoming_streams.next() => {
                    if let Some(listener_event) = listener_event {
                        // handle_listener_event(
                        //     listener_event.expect("listener event error"),
                        //     &known_peers,
                        //     &internal_event_sender).await
                        let listener_event = listener_event.expect("listener event error");
                        if let Some((upgrade, remote_addr)) = listener_event.into_upgrade() {
                            let (peer_id, stream) = upgrade.await.expect("upgrade failed");

                            if !process_stream(stream, peer_id, remote_addr, &known_peers, &internal_event_sender)
                                .await
                                .expect("error")
                            {
                                // trace!("Continuing Conn Manager. Cause: process_stream returned false");
                                // continue;
                            } else {
                                // trace!("Breaking Conn Manager. Cause: process_stream returned true");
                                // break;
                            }
                        } else {
                            trace!("Not an upgrade event");
                        }
                    } else {
                        trace!("Breaking Connection Manager. Cause: listener_event stream closed.");
                        break;
                    }
                },
            }
        }

        trace!("Connection Manager stopped.");
        Ok(())
    }
}

// #[inline]
// async fn handle_listener_event(
//     listener_event: ListenerEvent<ListenerUpgrade, io::Error>,
//     known_peers: &KnownPeerList,
//     internal_event_sender: &EventSender,
// ) {
//     // match listener_event {
//     //     ListenerEvent::NewAddress(address) => {
//     //         trace!("new address = {}", address);
//     //         return Ok(true);
//     //     }
//     //     ListenerEvent::AddressExpired(address) => {
//     //         trace!("address expired = {}", address);
//     //         return Ok(true);
//     //     }
//     //     ListenerEvent::Upgrade {
//     //         upgrade,
//     //         local_addr,
//     //         remote_addr,
//     //     } => {
//     //         trace!("upgrade");
//     //     }
//     //     ListenerEvent::Error(e) => {
//     //         trace!("error: {:?}", e);
//     //         return Err(io::Error::new(io::ErrorKind::InvalidInput, "error").into());
//     //     }
//     // }
//     if let Some((upgrade, remote_addr)) = listener_event.into_upgrade() {
//         let (peer_id, stream) = upgrade.await.expect("upgrade failed");

//         if !process_stream(stream, peer_id, remote_addr, known_peers, internal_event_sender)
//             .await
//             .expect("error")
//         {
//             // trace!("Continuing Conn Manager. Cause: process_stream returned false");
//             // continue;
//         } else {
//             // trace!("Breaking Conn Manager. Cause: process_stream returned true");
//             // break;
//         }
//     } else {
//         trace!("Not an upgrade event");
//     }
// }

#[inline]
async fn process_stream(
    stream: StreamMuxerBox,
    peer_id: PeerId,
    peer_address: Multiaddr,
    known_peers: &KnownPeerList,
    internal_event_sender: &EventSender,
) -> Result<bool, WorkerError> {
    let connection = match Connection::new(peer_id, peer_address, stream, Origin::Inbound) {
        Ok(conn) => conn,
        Err(e) => {
            warn!("Creating connection failed: {:?}.", e);

            return Ok(false);
        }
    };

    // TODO: check if this can be removed
    // if !known_peers.contains_address(&connection.peer_address) {
    //     warn!("Contacted by unknown address '{}'.", &connection.peer_address);
    //     warn!("Connection dropped.");

    //     return Ok(false);
    // }

    trace!(
        "Sucessfully established inbound connection to {} ({}).",
        connection.peer_address,
        connection.peer_id,
    );

    let internal_event_sender = internal_event_sender.clone();

    Ok(spawn_reader_writer(connection, internal_event_sender).await.is_ok())
}
