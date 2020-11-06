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

use super::{ConnectedPeerList, KnownPeerList};
use crate::{
    conns,
    interaction::{commands::Command, events::Event},
    RECONNECT_INTERVAL,
};

use bee_common::{shutdown::ShutdownListener, worker::Error as WorkerError};

use futures::{select, FutureExt, StreamExt};
use libp2p::{identity, Multiaddr, PeerId};
use log::*;

use std::{sync::atomic::Ordering, time::Duration};

type CommandReceiver = flume::Receiver<Command>;
type EventReceiver = flume::Receiver<Event>;
type EventSender = flume::Sender<Event>;

pub struct PeerManager {
    local_keys: identity::Keypair,
    command_receiver: flume::r#async::RecvStream<'static, Command>,
    event_sender: EventSender,
    internal_event_receiver: flume::r#async::RecvStream<'static, Event>,
    internal_event_sender: EventSender,
    known_peers: KnownPeerList,
    connected_peers: ConnectedPeerList,
    shutdown_listener: ShutdownListener,
}

impl PeerManager {
    pub fn new(
        local_keys: identity::Keypair,
        command_receiver: CommandReceiver,
        event_sender: EventSender,
        internal_event_receiver: EventReceiver,
        internal_event_sender: EventSender,
        known_peers: KnownPeerList,
        connected_peers: ConnectedPeerList,
        shutdown_listener: ShutdownListener,
    ) -> Self {
        trace!("Starting Peer Manager...");

        Self {
            local_keys,
            command_receiver: command_receiver.into_stream(),
            event_sender,
            internal_event_receiver: internal_event_receiver.into_stream(),
            internal_event_sender,
            known_peers,
            connected_peers,
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
            mut known_peers,
            mut connected_peers,
            shutdown_listener,
            ..
        } = self;

        let mut fused_shutdown_listener = shutdown_listener.fuse();

        loop {
            select! {
                _ = fused_shutdown_listener => {
                    trace!("Breaking Peer Manager. Cause: shutdown listener");
                    break;
                },
                command = command_receiver.next() => {
                    if !process_command(command, &local_keys, &mut known_peers, &mut connected_peers, &mut event_sender, &mut internal_event_sender).await? {
                        trace!("Breaking Peer Manager. Cause: process_command returned false");
                        break;
                    }
                },
                event = internal_event_receiver.next() => {
                    if !process_event(event, &local_keys, &mut known_peers, &mut connected_peers, &mut event_sender, &mut internal_event_sender).await? {
                        trace!("Breaking Peer Manager. Cause: process_event returned false");
                        break;
                    }
                },
            }
        }

        trace!("Peer Manager stopped.");
        Ok(())
    }
}

#[inline]
async fn process_command(
    command: Option<Command>,
    local_keys: &identity::Keypair,
    mut known_peers: &mut KnownPeerList,
    mut connected_peers: &mut ConnectedPeerList,
    event_sender: &mut EventSender,
    mut internal_event_sender: &mut EventSender,
) -> Result<bool, WorkerError> {
    let command = if let Some(command) = command {
        command
    } else {
        error!("Command channel unexpectedly closed.");
        return Ok(false);
    };

    trace!("Received {}.", command);

    match command {
        Command::AddEndpoint { address } => {
            add_endpoint(address, &mut known_peers, &mut internal_event_sender).await?;
        }

        Command::RemoveEndpoint { address } => {
            remove_endpoint(
                address,
                &mut known_peers,
                &mut connected_peers,
                &mut internal_event_sender,
            )
            .await?;
        }

        Command::DialPeer { endpoint_address } => {
            dial_peer(
                endpoint_address,
                local_keys,
                &mut known_peers,
                &mut connected_peers,
                &mut internal_event_sender,
            )
            .await?;
        }

        Command::DisconnectPeer { id } => {
            if disconnect_peer(&id, &mut connected_peers)? {
                event_sender
                    .send_async(Event::PeerDisconnected { id })
                    .await
                    .map_err(|e| WorkerError(Box::new(e)))?;
            }
        }

        Command::SendMessage { peer_id, message } => {
            send_message(&peer_id, message, &mut connected_peers).await?;
        }


        // /* Command::MarkDuplicate {
        //    *     duplicate_epid,
        //    *     original_epid,
        //    * } => {
        //    *     mark_duplicate(
        //    *         duplicate_epid,
        //    *         original_epid,
        //    *         &mut connected_peers,
        //    *         &mut internal_event_sender,
        //    *     )?;
        //    * } */
    }

    Ok(true)
}

#[inline]
async fn process_event(
    event: Option<Event>,
    local_keys: &identity::Keypair,
    known_peers: &mut KnownPeerList,
    connected_peers: &mut ConnectedPeerList,
    event_sender: &mut EventSender,
    mut internal_event_sender: &mut EventSender,
) -> Result<bool, WorkerError> {
    let event = if let Some(event) = event {
        event
    } else {
        error!("Event channel unexpectedly closed.");
        return Ok(false);
    };

    trace!("Received {}.", event);

    match event {
        Event::EndpointAdded { address } => {
            event_sender
                .send_async(Event::EndpointAdded { address })
                .await
                .map_err(|e| WorkerError(Box::new(e)))?;
        }

        Event::EndpointRemoved { address } => {
            event_sender
                .send_async(Event::EndpointRemoved { address })
                .await
                .map_err(|e| WorkerError(Box::new(e)))?;
        }

        Event::ConnectionEstablished {
            peer_id,
            endpoint_address,
            origin,
            message_sender,
        } => {
            if connected_peers.insert(peer_id.clone(), message_sender) {
                // new peer
                event_sender
                    .send_async(Event::PeerConnected {
                        id: peer_id,
                        endpoint_address,
                        origin,
                    })
                    .await
                    .map_err(|e| WorkerError(Box::new(e)))?
            } else {
                // already connected peer
                // TODO: drop that connection if the dialer doesn't drop it on his behalf
                log::info!("Dropping duplicate connection with: {}", peer_id);
            }
        }

        Event::ConnectionDropped {
            peer_id,
            endpoint_address,
            ..
        } => {
            // TODO: check, if the contact belonging to the dropped connection is still a liked peer
            if known_peers.contains_peer_id(&peer_id) {
                dial_peer(
                    endpoint_address,
                    &local_keys,
                    known_peers,
                    connected_peers,
                    &mut internal_event_sender,
                )
                .await?;
            }
        }

        Event::MessageReceived { peer_id, message } => event_sender
            .send_async(Event::MessageReceived { peer_id, message })
            .await
            .map_err(|e| WorkerError(Box::new(e)))?,

        Event::ReconnectTimerElapsed { endpoint_address } => {
            dial_peer(
                endpoint_address,
                &local_keys,
                known_peers,
                connected_peers,
                &mut internal_event_sender,
            )
            .await?;
        }
        _ => (),
    }

    Ok(true)
}
#[inline]
async fn add_endpoint(
    endpoint_address: Multiaddr,
    known_peers: &mut KnownPeerList,
    internal_event_sender: &mut EventSender,
) -> Result<bool, WorkerError> {
    if known_peers.insert_address(endpoint_address.clone()) {
        internal_event_sender
            .send_async(Event::EndpointAdded {
                address: endpoint_address,
            })
            .await
            .map_err(|e| WorkerError(Box::new(e)))?;

        Ok(true)
    } else {
        Ok(false)
    }
}

#[inline]
async fn remove_endpoint(
    endpoint_address: Multiaddr,
    known_peers: &mut KnownPeerList,
    connected_peers: &mut ConnectedPeerList,
    internal_event_sender: &mut EventSender,
) -> Result<bool, WorkerError> {
    if let Some(peer_id) = known_peers.remove_peer_by_address(&endpoint_address) {
        if let Some(peer_id) = peer_id {
            if connected_peers.remove(&peer_id) {
                trace!("Removed and disconnected peer {} at {}.", peer_id, endpoint_address);
            } else {
                trace!("Removed peer reached at {}.", endpoint_address);
            }
        } else {
            trace!("Removed peer reached at {}.", endpoint_address);
        }

        // TODO: set proper peer_id if possible
        internal_event_sender
            .send_async(Event::EndpointRemoved {
                address: endpoint_address,
            })
            .await
            .map_err(|e| WorkerError(Box::new(e)))?;

        Ok(true)
    } else {
        Ok(false)
    }
}

#[inline]
async fn dial_peer(
    endpoint_address: Multiaddr,
    local_keys: &identity::Keypair,
    known_peers: &KnownPeerList,
    connected_peers: &ConnectedPeerList,
    internal_event_sender: &mut EventSender,
) -> Result<bool, WorkerError> {
    if conns::dial_peer(
        endpoint_address.clone(),
        local_keys,
        internal_event_sender.clone(),
        connected_peers,
    )
    .await
    .is_ok()
    {
        Ok(true)
    } else {
        tokio::spawn(send_event_after_delay(
            Event::ReconnectTimerElapsed { endpoint_address },
            internal_event_sender.clone(),
        ));
        Ok(false)
    }
}

#[inline]
fn disconnect_peer(peer_id: &PeerId, connected_peers: &mut ConnectedPeerList) -> Result<bool, WorkerError> {
    // NOTE: removing the endpoint will drop the connection!
    Ok(connected_peers.remove(peer_id))
}

#[inline]
async fn send_event_after_delay(event: Event, internal_event_sender: EventSender) -> Result<(), WorkerError> {
    tokio::time::delay_for(Duration::from_secs(RECONNECT_INTERVAL.load(Ordering::Relaxed))).await;

    Ok(internal_event_sender
        .send_async(event)
        .await
        .map_err(|e| WorkerError(Box::new(e)))?)
}

#[inline]
async fn send_message(
    peer_id: &PeerId,
    message: Vec<u8>,
    connected_peers: &mut ConnectedPeerList,
) -> Result<bool, WorkerError> {
    Ok(connected_peers.send_message(message, peer_id).await?)
}
