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
    conns,
    interaction::{
        commands::Command,
        events::{Event, EventSender, InternalEvent, InternalEventReceiver, InternalEventSender},
    },
    ReadableId, RECONNECT_MILLIS,
};

use super::{errors::Error, BannedAddrList, BannedPeerList, PeerList};

use bee_common::{shutdown::ShutdownListener, worker::Error as WorkerError};

use futures::{select, FutureExt, StreamExt};
use libp2p::{identity, Multiaddr, PeerId};
use log::*;

use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

type CommandReceiver = flume::Receiver<Command>;

pub static NUM_COMMAND_PROCESSING_ERRORS: AtomicUsize = AtomicUsize::new(0);
pub static NUM_EVENT_PROCESSING_ERRORS: AtomicUsize = AtomicUsize::new(0);

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
        trace!("Peer Manager started.");

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
                    let command = if let Some(command) = command {
                        command
                    } else {
                        error!("Fatal: Command channel unexpectedly stopped.");
                        break;
                    };

                    if let Err(e) = process_command(
                        command,
                        &local_keys,
                        &mut peers,
                        &mut banned_addrs,
                        &mut banned_peers,
                        &mut event_sender,
                        &mut internal_event_sender
                    ).await {
                        error!("Error processing command. Error: {}", e);
                        NUM_COMMAND_PROCESSING_ERRORS.fetch_add(1, Ordering::Relaxed);
                        continue;
                    }
                },
                internal_event = internal_event_receiver.next() => {
                    let internal_event = if let Some(internal_event) = internal_event {
                        internal_event
                    } else {
                        error!("Fatal: Internal event channel unexpectedly stopped.");
                        break;
                    };

                    if let Err(e) = process_internal_event(
                        internal_event,
                        &local_keys,
                        &peers,
                        &banned_addrs,
                        &banned_peers,
                        &mut event_sender,
                        &mut internal_event_sender
                    ).await {
                        error!("Error processing internal event. Error: {}", e);
                        NUM_EVENT_PROCESSING_ERRORS.fetch_add(1, Ordering::Relaxed);
                        continue;
                    }
                },
            }
        }

        trace!("Peer Manager stopped.");
        Ok(())
    }
}

async fn process_command(
    command: Command,
    local_keys: &identity::Keypair,
    peers: &mut PeerList,
    banned_addrs: &mut BannedAddrList,
    banned_peers: &mut BannedPeerList,
    event_sender: &mut EventSender,
    internal_event_sender: &mut InternalEventSender,
) -> Result<(), Error> {
    trace!("Received {:?}.", command);

    match command {
        Command::ConnectPeer { address, id } => {
            connect_peer(
                address,
                Some(id),
                local_keys,
                &peers,
                &banned_addrs,
                &banned_peers,
                &internal_event_sender,
            )
            .await
        }
        Command::ConnectUnknownPeer { address } => {
            connect_peer(
                address,
                None,
                local_keys,
                &peers,
                &banned_addrs,
                &banned_peers,
                &internal_event_sender,
            )
            .await
        }
        Command::DisconnectPeer { id } => {
            if disconnect_peer(&id, peers) {
                event_sender
                    .send_async(Event::PeerDisconnected { id })
                    .await
                    .map_err(|_| Error::EventSendFailure("PeerDisconnected"))?;
            } else {
                return Err(Error::DisconnectPeerFailure(id.readable()));
            }
        }
        Command::SendMessage { message, to } => {
            send_message(message, &to, peers).await?;
        }
        Command::BanAddress { address } => {
            if !banned_addrs.insert(address.to_string()) {
                return Err(Error::AddressAlreadyBanned(address));
            } else {
                event_sender
                    .send_async(Event::AddressBanned { address })
                    .await
                    .map_err(|_| Error::EventSendFailure("AddressBanned"))?;
            }
        }
        Command::BanPeer { id } => {
            if !banned_peers.insert(id.clone()) {
                return Err(Error::PeerAlreadyBanned(id.readable()));
            } else {
                event_sender
                    .send_async(Event::PeerBanned { id })
                    .await
                    .map_err(|_| Error::EventSendFailure("PeerBanned"))?;
            }
        }
        Command::UnbanAddress { address } => {
            if !banned_addrs.remove(&address.to_string()) {
                return Err(Error::AddressAlreadyUnbanned(address));
            }
        }
        Command::UnbanPeer { id } => {
            if !banned_peers.remove(&id) {
                return Err(Error::PeerAlreadyUnbanned(id.readable()));
            }
        }
    }

    Ok(())
}

#[inline]
async fn process_internal_event(
    internal_event: InternalEvent,
    local_keys: &identity::Keypair,
    peers: &PeerList,
    banned_addrs: &BannedAddrList,
    banned_peers: &BannedPeerList,
    event_sender: &mut EventSender,
    internal_event_sender: &InternalEventSender,
) -> Result<(), Error> {
    trace!("Received {:?}.", internal_event);

    match internal_event {
        InternalEvent::ConnectionEstablished {
            peer_id,
            peer_address,
            message_sender,
            ..
        } => {
            if peers.insert_connected_peer(peer_id.clone(), message_sender) {
                // publish
                event_sender
                    .send_async(Event::PeerConnected {
                        id: peer_id,
                        address: peer_address,
                    })
                    .await
                    .map_err(|_| Error::EventSendFailure("PeerConnected"))?;
            } else {
                // FIXME
                unreachable!("already connected peer");
            }
        }

        InternalEvent::ConnectionDropped { peer_id, peer_address } => {
            // NOTE: if the peerlist still contains the peer_id, that means that we still like to keep the connection.
            if peers.contains_peer(&peer_id) {
                connect_peer(
                    peer_address,
                    Some(peer_id),
                    &local_keys,
                    &peers,
                    &banned_addrs,
                    &banned_peers,
                    &internal_event_sender,
                )
                .await;
            }
        }

        InternalEvent::MessageReceived { message, from } => event_sender
            .send_async(Event::MessageReceived { message, from })
            .await
            .map_err(|_| Error::EventSendFailure("MessageReceived"))?,

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
            .await;
        }
        _ => (),
    }

    Ok(())
}

async fn connect_peer(
    address: Multiaddr,
    id: Option<PeerId>,
    local_keys: &identity::Keypair,
    peers: &PeerList,
    banned_addrs: &BannedAddrList,
    banned_peers: &BannedPeerList,
    internal_event_sender: &InternalEventSender,
) {
    if let Err(e) = conns::dial(
        address.clone(),
        id.clone(),
        local_keys,
        internal_event_sender,
        peers,
        banned_addrs,
        banned_peers,
    )
    .await
    {
        warn!("Failed connecting to peer. Error: {}", e);

        // Only attempt to reconnect if the dialing itself failed for some reason, but for any other error that might
        // have happend
        if let conns::Error::DialingFailed(_) = e {
            tokio::spawn(send_reconnect_event_after_delay(
                InternalEvent::ReconnectScheduled {
                    peer_id: id,
                    peer_address: address,
                },
                internal_event_sender.clone(),
            ));
        }
    }
}

#[inline]
fn disconnect_peer(peer_id: &PeerId, peers: &mut PeerList) -> bool {
    peers.remove_peer(peer_id)
}

#[inline]
async fn send_reconnect_event_after_delay(
    internal_event: InternalEvent,
    internal_event_sender: InternalEventSender,
) -> Result<(), Error> {
    // TODO: should we randomize this a bit?
    tokio::time::delay_for(Duration::from_millis(RECONNECT_MILLIS.load(Ordering::Relaxed))).await;

    Ok(internal_event_sender
        .send_async(internal_event)
        .await
        .map_err(|_| Error::InternalEventSendFailure("ReconnectScheduled"))?)
}

#[inline]
async fn send_message(message: Vec<u8>, to: &PeerId, peers: &mut PeerList) -> Result<(), Error> {
    peers.send_message(message, to).await
}
