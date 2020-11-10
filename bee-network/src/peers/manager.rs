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

use super::{BannedAddrList, BannedPeerList, PeerList};
use crate::{
    conns,
    interaction::{
        commands::Command,
        events::{Event, EventSender, InternalEvent, InternalEventReceiver, InternalEventSender},
    },
    RECONNECT_MILLIS,
};

use bee_common::{shutdown::ShutdownListener, worker::Error as WorkerError};

use futures::{select, FutureExt, StreamExt};
use libp2p::{identity, Multiaddr, PeerId};
use log::*;

use std::{sync::atomic::Ordering, time::Duration};

type CommandReceiver = flume::Receiver<Command>;

pub struct PeerManager {
    local_keys: identity::Keypair,
    command_receiver: flume::r#async::RecvStream<'static, Command>,
    event_sender: EventSender,
    internal_event_receiver: flume::r#async::RecvStream<'static, InternalEvent>,
    internal_event_sender: InternalEventSender,
    peers: PeerList,
    banned_addrs: BannedAddrList,
    banned_peers: BannedPeerList,
    shutdown_listener: ShutdownListener,
}

impl PeerManager {
    pub fn new(
        local_keys: identity::Keypair,
        command_receiver: CommandReceiver,
        event_sender: EventSender,
        internal_event_receiver: InternalEventReceiver,
        internal_event_sender: InternalEventSender,
        peers: PeerList,
        banned_addrs: BannedAddrList,
        banned_peers: BannedPeerList,
        shutdown_listener: ShutdownListener,
    ) -> Self {
        trace!("Starting Peer Manager...");

        Self {
            local_keys,
            command_receiver: command_receiver.into_stream(),
            event_sender,
            internal_event_receiver: internal_event_receiver.into_stream(),
            internal_event_sender,
            peers,
            banned_addrs,
            banned_peers,
            shutdown_listener,
        }
    }

    pub async fn run(self) -> Result<(), WorkerError> {
        trace!("Peer Manager running...");

        let PeerManager {
            local_keys,
            mut command_receiver,
            mut event_sender,
            mut internal_event_receiver,
            mut internal_event_sender,
            mut peers,
            mut banned_addrs,
            mut banned_peers,
            shutdown_listener,
        } = self;

        let mut fused_shutdown_listener = shutdown_listener.fuse();

        loop {
            select! {
                _ = fused_shutdown_listener => {
                    trace!("Peer Manager received shutdown signal.");
                    break;
                },
                command = command_receiver.next() => {
                    if !process_command(
                        command,
                        &local_keys,
                        &mut peers,
                        &mut banned_addrs,
                        &mut banned_peers,
                        &mut event_sender,
                        &mut internal_event_sender
                    ).await? {
                        error!("Error processing command.");
                        // TODO: count processing failures
                    }
                },
                event = internal_event_receiver.next() => {
                    if !process_internal_event(
                        event,
                        &local_keys,
                        &peers,
                        &banned_addrs,
                        &banned_peers,
                        &mut event_sender,
                        &mut internal_event_sender
                    ).await? {
                        error!("Error processing event.");
                        // TODO: count processing failures
                    }
                },
            }
        }

        trace!("Peer Manager stopped.");
        Ok(())
    }
}

async fn process_command(
    command: Option<Command>,
    local_keys: &identity::Keypair,
    peers: &mut PeerList,
    banned_addrs: &mut BannedAddrList,
    banned_peers: &mut BannedPeerList,
    event_sender: &mut EventSender,
    internal_event_sender: &mut InternalEventSender,
) -> Result<bool, WorkerError> {
    let command = if let Some(command) = command {
        command
    } else {
        error!("Command channel unexpectedly closed.");
        return Ok(false);
    };

    trace!("Received {:?}.", command);

    match command {
        Command::ConnectPeer { address, id } => {
            connect_peer(
                address,
                id,
                local_keys,
                &peers,
                &banned_addrs,
                &banned_peers,
                &internal_event_sender,
            )
            .await?;
        }
        Command::ConnectUnknownPeer { address: _ } => todo!("connect unkown peer"),
        Command::DisconnectPeer { id } => {
            if disconnect_peer(&id, peers)? {
                event_sender
                    .send_async(Event::PeerDisconnected { id })
                    .await
                    .map_err(|e| WorkerError(Box::new(e)))?;
            }
        }
        Command::SendMessage { message, to } => {
            send_message(message, &to, peers).await?;
        }
        Command::BanIp { ip: _ } => todo!("ban ip"),
        Command::BanPeer { id: _ } => todo!("ban id"),
    }

    Ok(true)
}

#[inline]
async fn process_internal_event(
    internal_event: Option<InternalEvent>,
    local_keys: &identity::Keypair,
    peers: &PeerList,
    banned_addrs: &BannedAddrList,
    banned_peers: &BannedPeerList,
    event_sender: &mut EventSender,
    internal_event_sender: &InternalEventSender,
) -> Result<bool, WorkerError> {
    let internal_event = if let Some(internal_event) = internal_event {
        internal_event
    } else {
        error!("Event channel unexpectedly closed.");
        return Ok(false);
    };

    trace!("Received {:?}.", internal_event);

    match internal_event {
        InternalEvent::ConnectionEstablished {
            peer_id,
            peer_address,
            origin,
            message_sender,
        } => {
            if peers.insert_connected_peer(peer_id.clone(), message_sender) {
                event_sender
                    .send_async(Event::PeerConnected {
                        id: peer_id,
                        address: peer_address,
                        origin,
                    })
                    .await
                    .map_err(|e| WorkerError(Box::new(e)))?
            } else {
                unreachable!("already connected peer")
            }
        }

        InternalEvent::ConnectionDropped { peer_id, peer_address } => {
            // NOTE: if the peerlist still contains the peer_id, that means that we still like to keep the connection.
            if peers.contains_peer(&peer_id) {
                connect_peer(
                    peer_address,
                    peer_id,
                    &local_keys,
                    &peers,
                    &banned_addrs,
                    &banned_peers,
                    &internal_event_sender,
                )
                .await?;
            }
        }

        InternalEvent::MessageReceived { message, from } => event_sender
            .send_async(Event::MessageReceived { message, from })
            .await
            .map_err(|e| WorkerError(Box::new(e)))?,

        InternalEvent::ReconnectScheduled { peer_id, peer_address } => {
            connect_peer(
                peer_address,
                peer_id,
                &local_keys,
                &peers,
                &banned_addrs,
                &banned_peers,
                &internal_event_sender,
            )
            .await?;
        }
        _ => (),
    }

    Ok(true)
}

async fn connect_peer(
    address: Multiaddr,
    id: PeerId,
    local_keys: &identity::Keypair,
    peers: &PeerList,
    banned_addrs: &BannedAddrList,
    banned_peers: &BannedPeerList,
    internal_event_sender: &InternalEventSender,
) -> Result<(), WorkerError> {
    // Dial the peer
    if conns::dial(
        address.clone(),
        id.clone(),
        local_keys,
        internal_event_sender.clone(),
        peers,
        banned_addrs,
        banned_peers,
    )
    .await
    .is_ok()
    {
    } else {
        tokio::spawn(send_event_after_delay(
            InternalEvent::ReconnectScheduled {
                peer_id: id,
                peer_address: address,
            },
            internal_event_sender.clone(),
        ));
    }
    Ok(())
}

#[inline]
fn disconnect_peer(peer_id: &PeerId, peers: &mut PeerList) -> Result<bool, WorkerError> {
    Ok(peers.remove_peer(peer_id))
}

#[inline]
async fn send_event_after_delay(
    internal_event: InternalEvent,
    internal_event_sender: InternalEventSender,
) -> Result<(), WorkerError> {
    // TODO: should we randomize this a bit?
    tokio::time::delay_for(Duration::from_secs(RECONNECT_MILLIS.load(Ordering::Relaxed))).await;

    Ok(internal_event_sender
        .send_async(internal_event)
        .await
        .map_err(|e| WorkerError(Box::new(e)))?)
}

#[inline]
async fn send_message(message: Vec<u8>, to: &PeerId, peers: &mut PeerList) -> Result<bool, WorkerError> {
    Ok(peers.send_message(message, to).await?)
}

// #[inline]
// async fn add_endpoint(
//     endpoint_address: Multiaddr,
//     known_peers: &mut KnownPeerList,
//     internal_event_sender: &mut EventSender,
// ) -> Result<bool, WorkerError> {
//     if known_peers.insert_address(endpoint_address.clone()) {
//         internal_event_sender
//             .send_async(Event::EndpointAdded {
//                 address: endpoint_address,
//             })
//             .await
//             .map_err(|e| WorkerError(Box::new(e)))?;

//         Ok(true)
//     } else {
//         Ok(false)
//     }
// }

// #[inline]
// async fn remove_endpoint(
//     endpoint_address: Multiaddr,
//     known_peers: &mut KnownPeerList,
//     connected_peers: &mut ConnectedPeerList,
//     internal_event_sender: &mut EventSender,
// ) -> Result<bool, WorkerError> {
//     if let Some(peer_id) = known_peers.remove_peer_by_address(&endpoint_address) {
//         if let Some(peer_id) = peer_id {
//             if connected_peers.remove(&peer_id) {
//                 trace!("Removed and disconnected peer {} at {}.", peer_id, endpoint_address);
//             } else {
//                 trace!("Removed peer reached at {}.", endpoint_address);
//             }
//         } else {
//             trace!("Removed peer reached at {}.", endpoint_address);
//         }

//         // TODO: set proper peer_id if possible
//         internal_event_sender
//             .send_async(Event::EndpointRemoved {
//                 address: endpoint_address,
//             })
//             .await
//             .map_err(|e| WorkerError(Box::new(e)))?;

//         Ok(true)
//     } else {
//         Ok(false)
//     }
// }
