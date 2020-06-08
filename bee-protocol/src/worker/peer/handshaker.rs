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
    config::slice_eq,
    message::{
        messages_supported_version, tlv_from_bytes, tlv_into_bytes, Handshake, Header, Message, MESSAGES_VERSIONS,
    },
    peer::Peer,
    protocol::Protocol,
    worker::PeerWorker,
};

use bee_network::{
    Address,
    Command::{Disconnect, SendMessage},
    Network, Origin, Port,
};
use bee_tangle::tangle;

use async_std::{net::SocketAddr, task::spawn};
use futures::{
    channel::{mpsc, oneshot},
    future::FutureExt,
    select,
    stream::StreamExt,
};
use log::{debug, error, info, warn};

use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug)]
pub(crate) enum HandshakeError {
    InvalidTimestampDiff(i64),
    CoordinatorMismatch,
    MwmMismatch(u8, u8),
    UnsupportedVersion(u8),
    PortMismatch(u16, u16),
    UnboundPeer,
    AlreadyHandshaked,
}

#[derive(Debug)]
pub(crate) enum PeerHandshakerWorkerError {}

enum PeerReadState {
    Header,
    Payload(Header),
}

struct PeerReadContext {
    state: PeerReadState,
    buffer: Vec<u8>,
}

enum HandshakeStatus {
    Awaiting,
    Done,
    Duplicate,
}

pub struct PeerHandshakerWorker {
    network: Network,
    peer: Arc<Peer>,
    status: HandshakeStatus,
}

impl PeerHandshakerWorker {
    pub(crate) fn new(network: Network, peer: Arc<Peer>) -> Self {
        Self {
            network,
            peer,
            status: HandshakeStatus::Awaiting,
        }
    }

    pub async fn run(mut self, receiver: mpsc::Receiver<Vec<u8>>, shutdown: oneshot::Receiver<()>) {
        info!("[{}] Running.", self.peer.address);

        // TODO should we have a first check if already connected ?

        let mut context = PeerReadContext {
            state: PeerReadState::Header,
            buffer: Vec::new(),
        };
        let mut receiver_fused = receiver.fuse();
        let mut shutdown_fused = shutdown.fuse();

        // This is the only message not using a SenderWorker because they are not running yet (awaiting handshake)
        if let Err(e) = self
            .network
            .send(SendMessage {
                epid: self.peer.epid,
                bytes: tlv_into_bytes(Handshake::new(
                    self.network.config().binding_port(),
                    &Protocol::get().config.coordinator.public_key_bytes,
                    Protocol::get().config.mwm,
                    &MESSAGES_VERSIONS,
                )),
                responder: None,
            })
            .await
        {
            // TODO then what ?
            warn!("[{}] Failed to send handshake: {:?}.", self.peer.address, e);
        }

        loop {
            select! {
                event = receiver_fused.next() => {
                    if let Some(event) = event {
                        context = self.message_handler(context, event).await;
                        match self.status {
                            HandshakeStatus::Done | HandshakeStatus::Duplicate => break,
                            _ => continue
                        }
                    }
                },
                _ = shutdown_fused => {
                    break;
                }
            }
        }

        match self.status {
            HandshakeStatus::Done => {
                spawn(
                    PeerWorker::new(
                        Protocol::get()
                            .peer_manager
                            .handshaked_peers
                            .get(&self.peer.epid)
                            .unwrap()
                            .value()
                            .clone(),
                    )
                    .run(receiver_fused, shutdown_fused),
                );
            }
            HandshakeStatus::Duplicate => {
                info!("[{}] Closing duplicate connection.", self.peer.epid);
                if let Err(e) = self
                    .network
                    .send(Disconnect {
                        epid: self.peer.epid,
                        responder: None,
                    })
                    .await
                {
                    warn!("[{}] Disconnecting peer failed: {}.", self.peer.epid, e);
                }
            }
            _ => (),
        }

        info!("[{}] Stopped.", self.peer.address);
    }

    pub(crate) fn validate_handshake(&mut self, handshake: Handshake) -> Result<Address, HandshakeError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Clock may have gone backwards")
            .as_millis() as u64;

        if ((timestamp - handshake.timestamp) as i64).abs() as u64 > Protocol::get().config.handshake_window * 1000 {
            return Err(HandshakeError::InvalidTimestampDiff(
                ((timestamp - handshake.timestamp) as i64).abs(),
            ));
        }

        if !slice_eq(
            &Protocol::get().config.coordinator.public_key_bytes,
            &handshake.coordinator,
        ) {
            return Err(HandshakeError::CoordinatorMismatch);
        }

        if Protocol::get().config.mwm != handshake.minimum_weight_magnitude {
            return Err(HandshakeError::MwmMismatch(
                Protocol::get().config.mwm,
                handshake.minimum_weight_magnitude,
            ));
        }

        if let Err(version) = messages_supported_version(&handshake.supported_versions) {
            return Err(HandshakeError::UnsupportedVersion(version));
        }

        let address = match self.peer.origin {
            Origin::Outbound => {
                if self.peer.address.port() != Port(handshake.port) {
                    return Err(HandshakeError::PortMismatch(*self.peer.address.port(), handshake.port));
                }

                self.peer.address
            }
            Origin::Inbound => {
                // TODO check if whitelisted

                Address::from(SocketAddr::new(self.peer.address.ip(), handshake.port))
            }
            Origin::Unbound => return Err(HandshakeError::UnboundPeer),
        };

        for peer in Protocol::get().peer_manager.handshaked_peers.iter() {
            if peer.address == address {
                self.status = HandshakeStatus::Duplicate;
                return Err(HandshakeError::AlreadyHandshaked);
            }
        }

        Ok(address)
    }

    async fn process_message(&mut self, header: &Header, bytes: &[u8]) -> Result<(), PeerHandshakerWorkerError> {
        if let Handshake::ID = header.message_type {
            debug!("[{}] Reading Handshake...", self.peer.address);
            match tlv_from_bytes::<Handshake>(&header, bytes) {
                Ok(handshake) => match self.validate_handshake(handshake) {
                    Ok(address) => {
                        info!("[{}] Handshake completed.", self.peer.address);

                        Protocol::get().peer_manager.handshake(&self.peer.epid, address);

                        Protocol::send_heartbeat(
                            self.peer.epid,
                            *tangle().get_solid_milestone_index(),
                            *tangle().get_snapshot_milestone_index(),
                        )
                        .await;

                        Protocol::request_last_milestone(Some(self.peer.epid));
                        Protocol::trigger_milestone_solidification().await;

                        self.status = HandshakeStatus::Done;
                    }
                    Err(e) => {
                        warn!("[{}] Handshaking failed: {:?}.", self.peer.address, e);
                    }
                },
                Err(e) => {
                    warn!("[{}] Reading Handshake failed: {:?}.", self.peer.address, e);

                    Protocol::get().metrics.invalid_messages_received_inc();
                }
            }
        } else {
            warn!("[{}] Ignoring messages until fully handshaked.", self.peer.address);

            Protocol::get().metrics.invalid_messages_received_inc();
        }

        Ok(())
    }

    async fn message_handler(&mut self, mut context: PeerReadContext, mut bytes: Vec<u8>) -> PeerReadContext {
        let mut offset = 0;
        let mut remaining = true;

        if context.buffer.is_empty() {
            context.buffer = bytes;
        } else {
            context.buffer.append(&mut bytes);
        }

        while remaining {
            context.state = match context.state {
                PeerReadState::Header => {
                    if offset + 3 <= context.buffer.len() {
                        debug!("[{}] Reading Header...", self.peer.address);
                        let header = Header::from_bytes(&context.buffer[offset..offset + 3]);
                        offset += 3;

                        PeerReadState::Payload(header)
                    } else {
                        remaining = false;

                        PeerReadState::Header
                    }
                }
                PeerReadState::Payload(header) => {
                    if (offset + header.message_length as usize) <= context.buffer.len() {
                        if let Err(e) = self
                            .process_message(
                                &header,
                                &context.buffer[offset..offset + header.message_length as usize],
                            )
                            .await
                        {
                            error!("[{}] Processing message failed: {:?}.", self.peer.address, e);
                        }

                        offset += header.message_length as usize;

                        PeerReadState::Header
                    } else {
                        remaining = false;

                        PeerReadState::Payload(header)
                    }
                }
            };
        }

        PeerReadContext {
            state: context.state,
            buffer: context.buffer[offset..].to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {}
