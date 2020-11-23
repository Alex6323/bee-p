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
    conns::Error as ConnError,
    interaction::{
        commands::Command,
        events::{Event, EventSender, InternalEvent, InternalEventReceiver, InternalEventSender},
    },
    peers::{PeerRelation, PeerState, Repeat},
    ShortId, RECONNECT_MILLIS,
};

use super::{errors::Error, BannedAddrList, BannedPeerList, PeerInfo, PeerList};

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
            event_sender,
            mut internal_event_receiver,
            internal_event_sender,
            peers,
            banned_addrs,
            banned_peers,
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
                        &peers,
                        &banned_addrs,
                        &banned_peers,
                        &event_sender,
                        &internal_event_sender
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
                        &event_sender,
                        &internal_event_sender
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
    peers: &PeerList,
    banned_addrs: &BannedAddrList,
    banned_peers: &BannedPeerList,
    event_sender: &EventSender,
    internal_event_sender: &InternalEventSender,
) -> Result<(), Error> {
    trace!("Received {:?}.", command);

    match command {
        Command::AddPeer {
            id,
            address,
            alias,
            relation,
        } => add_peer(id, address, alias, relation, peers, event_sender).await?,
        Command::RemovePeer { id } => remove_peer(id, peers, event_sender).await?,
        Command::ConnectPeer { id } => {
            connect_peer(
                id,
                local_keys,
                peers,
                banned_addrs,
                banned_peers,
                internal_event_sender,
                event_sender,
            )
            .await?;
        }
        Command::DisconnectPeer { id } => {
            disconnect_peer(id, peers, event_sender).await?;
        }
        Command::DialAddress { address } => {
            dial_address(
                address,
                local_keys,
                &peers,
                &banned_addrs,
                &banned_peers,
                &internal_event_sender,
            )
            .await
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
                return Err(Error::PeerAlreadyBanned(id.short()));
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
                return Err(Error::PeerAlreadyUnbanned(id.short()));
            }
        }
        Command::UpdateRelation { id, relation } => todo!("UpdateRelation"),
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
    event_sender: &EventSender,
    internal_event_sender: &InternalEventSender,
) -> Result<(), Error> {
    trace!("Received {:?}.", internal_event);

    match internal_event {
        InternalEvent::ConnectionEstablished {
            peer_id,
            peer_info,
            origin,
            message_sender,
            ..
        } => {
            // TODO: if the connection was outbound via `DialAddress` (hence cannot be in the peerlist yet)
            // then we insert it as an `is_unknown` peer, that manually has to be upgraded to `known` for
            // automatic reconnects
            // if !peers.contains(&peer_id) && origin.is_outbound() {
            //     peers.insert(
            //         peer_id.clone(),
            //         PeerInfo {
            //             address: peer_address.clone(),
            //             alias: None,
            //             relation: PeerRelation::Unknown,
            //         },
            //     );
            // }

            peers.update_state(&peer_id, PeerState::Connected(message_sender));

            // // TODO!!!
            // if peers.insert(
            //     peer_id.clone(),
            //     PeerInfo {
            //         relation,
            //         connection: Some(message_sender),
            //     },
            // ) {
            //     // publish
            //     event_sender
            //         .send_async(Event::PeerConnected {
            //             id: peer_id,
            //             address: peer_address,
            //             alias,
            //         })
            //         .await
            //         .map_err(|_| Error::EventSendFailure("PeerConnected"))?;
            // } else {
            //     // FIXME
            //     unreachable!("already connected peer");
            // }
        }

        InternalEvent::ConnectionDropped { peer_id } => {
            if let Some(peer_info) = peers.get_info(&peer_id) {
                match peer_info.relation {
                    PeerRelation::Known => {
                        //
                    }
                    _ => {
                        //
                    }
                }
            }
            // NOTE: if the peer is known, then that means that we still like to keep the connection.
            if peers.is_known(&peer_id) {
                peers.update_state(
                    &peer_id,
                    PeerState::Connecting {
                        remaining_attempts: Repeat::Forever,
                    },
                );
                // TODO: try to immediatedly reconnect, and spawn the reconnector if that fails

                // connect_peer(
                //     Some(peer_id),
                //     peer_address,
                //     &local_keys,
                //     &peers,
                //     &banned_addrs,
                //     &banned_peers,
                //     &internal_event_sender,
                // )
                // .await;
            }

            if peers.is_unknown(&peer_id) {
                // TODO: should we remove it from the peer list, or allow manual reconnect via its peer id?
            }
        }

        InternalEvent::MessageReceived { message, from } => event_sender
            .send_async(Event::MessageReceived { message, from })
            .await
            .map_err(|_| Error::EventSendFailure("MessageReceived"))?,

        InternalEvent::ReconnectScheduled { peer_id } => {
            if peers.is_known(&peer_id) {
                if let Some(peer_info) = peers.get_info(&peer_id) {
                    connect_peer(
                        peer_id,
                        &local_keys,
                        &peers,
                        &banned_addrs,
                        &banned_peers,
                        &internal_event_sender,
                        &event_sender,
                    )
                    .await;
                } else {
                    trace!("Peer was removed in the mean-time. Stopping reconnects");
                }
            } else {
                // peer was set to unknown or discovered in the mean-time
            }
        }
        _ => (),
    }

    Ok(())
}

#[inline]
async fn add_peer(
    id: PeerId,
    address: Multiaddr,
    alias: Option<String>,
    relation: PeerRelation,
    peers: &PeerList,
    event_sender: &EventSender,
) -> Result<(), Error> {
    let info = PeerInfo {
        address,
        alias,
        relation,
    };

    // If the insert fails for some reason, we get the peer info back.
    if let Err((info, e)) = peers.insert(id.clone(), info) {
        // Tell the user that the command failed.
        event_sender
            .send_async(Event::CommandFailed {
                command: Command::AddPeer {
                    id,
                    address: info.address,
                    alias: info.alias,
                    relation: info.relation,
                },
            })
            .await
            .map_err(|_| Error::EventSendFailure("CommandFailed"))?;

        return Err(e);
    }

    // Tell the user that the command succeeded.
    event_sender
        .send_async(Event::PeerAdded { id })
        .await
        .map_err(|_| Error::EventSendFailure("PeerAdded"))?;

    Ok(())
}

#[inline]
async fn remove_peer(id: PeerId, peers: &PeerList, event_sender: &EventSender) -> Result<(), Error> {
    match peers.remove(&id) {
        Err(e) => {
            // Tell the user that the command failed.
            event_sender
                .send_async(Event::CommandFailed {
                    command: Command::RemovePeer { id },
                })
                .await
                .map_err(|_| Error::EventSendFailure("CommandFailed"))?;

            Err(e)
        }
        Ok(peer_info) => {
            // Tell the user that the command succeeded.
            event_sender
                .send_async(Event::PeerRemoved { id })
                .await
                .map_err(|_| Error::EventSendFailure("PeerRemoved"))?;

            Ok(())
        }
    }
}

#[inline]
async fn connect_peer(
    id: PeerId,
    local_keys: &identity::Keypair,
    peers: &PeerList,
    banned_addrs: &BannedAddrList,
    banned_peers: &BannedPeerList,
    internal_event_sender: &InternalEventSender,
    event_sender: &EventSender,
) -> Result<(), Error> {
    if let Err(e) = conns::dial_peer(id, local_keys, internal_event_sender, peers, banned_addrs, banned_peers).await {
        // If only the dialing failed, then the peer might only be temporarily offline, and we should automatically try
        // again.
        if let ConnError::DialingFailed(_) = e {
            // TODO: make this more sophisticated! Maybe something like a `ConnectionWatcher` that makes sure all
            // `known` peers keep connected.

            // tokio::spawn(send_reconnect_event_after_delay(
            //     InternalEvent::ReconnectScheduled { peer_id: id },
            //     internal_event_sender.clone(),
            // ));
        }
    }

    Ok(())
}

#[inline]
async fn disconnect_peer(id: PeerId, peers: &PeerList, event_sender: &EventSender) -> Result<(), Error> {
    // if !peers.update_state(peer_id, PeerState::Disconnected) {}

    // if let Err(e) = peers.remove(peer_id) {
    //     todo!()
    // } else {
    //     event_sender
    //         .send_async(Event::PeerDisconnected { id })
    //         .await
    //         .map_err(|_| Error::EventSendFailure("PeerDisconnected"))?;

    //     return Err(Error::DisconnectPeerFailure(id.short()));
    // }
    todo!()
}

async fn dial_address(
    peer_address: Multiaddr,
    local_keys: &identity::Keypair,
    peers: &PeerList,
    banned_addrs: &BannedAddrList,
    banned_peers: &BannedPeerList,
    internal_event_sender: &InternalEventSender,
) {
    if let Err(e) = conns::dial_address(
        peer_address,
        local_keys,
        internal_event_sender,
        peers,
        banned_addrs,
        banned_peers,
    )
    .await
    {
        error!("Failed dialing address. Error: {:?}", e);

        if let conns::Error::DialingFailed(_) = e {
            // todo!("DialingFailed")
        }
    }
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
async fn send_message(message: Vec<u8>, to: &PeerId, peers: &PeerList) -> Result<(), Error> {
    peers.send_message(message, to).await
}

#[inline]
async fn handle_connection_established() -> Result<(), Error> {
    todo!()
}

#[inline]
async fn handle_connection_dropped(peer_id: &PeerId) -> Result<(), Error> {
    todo!()
}
