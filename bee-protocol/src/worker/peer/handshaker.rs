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
    event::HandshakeCompleted,
    message::{
        messages_supported_version, tlv_from_bytes, tlv_into_bytes, Handshake, Header, Message, MESSAGES_VERSIONS,
    },
    peer::Peer,
    protocol::Protocol,
    tangle::tangle,
    worker::{peer::MessageHandler, PeerWorker},
};

use bee_network::{
    Address,
    Command::{Disconnect, SendMessage},
    Network, Origin, Port,
};

use async_std::{net::SocketAddr, task::spawn};
use futures::{
    channel::{mpsc, oneshot},
    future::FutureExt,
    stream::StreamExt,
};
use log::{error, info, trace, warn};

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

    pub async fn run(mut self, receiver: mpsc::UnboundedReceiver<Vec<u8>>, shutdown: oneshot::Receiver<()>) {
        info!("[{}] Running.", self.peer.address);

        // TODO should we have a first check if already connected ?

        let receiver_fused = receiver.fuse();
        let shutdown_fused = shutdown.fuse();

        // This is the only message not using a SenderWorker because they are not running yet (awaiting handshake)
        if let Err(e) = self
            .network
            .send(SendMessage {
                epid: self.peer.epid,
                bytes: tlv_into_bytes(Handshake::new(
                    *self.network.config().binding_port(),
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

        let mut message_handler = MessageHandler::new(receiver_fused, shutdown_fused, self.peer.address);

        while let Some((header, bytes)) = message_handler.fetch_message().await {
            if let Err(e) = self.process_message(&header, bytes).await {
                error!("[{}] Processing message failed: {:?}.", self.peer.address, e);
            }
            if let HandshakeStatus::Done | HandshakeStatus::Duplicate = self.status {
                break;
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
                    .run(message_handler),
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
            trace!("[{}] Reading Handshake...", self.peer.address);
            match tlv_from_bytes::<Handshake>(&header, bytes) {
                Ok(handshake) => match self.validate_handshake(handshake) {
                    Ok(address) => {
                        info!("[{}] Handshake completed.", self.peer.address);

                        Protocol::get().peer_manager.handshake(&self.peer.epid, address).await;

                        Protocol::get().bus.dispatch(HandshakeCompleted(address));

                        Protocol::send_heartbeat(
                            self.peer.epid,
                            tangle().get_last_solid_milestone_index(),
                            tangle().get_snapshot_milestone_index(),
                            tangle().get_last_milestone_index(),
                        );

                        Protocol::request_last_milestone(Some(self.peer.epid));

                        self.status = HandshakeStatus::Done;
                    }
                    Err(e) => {
                        warn!("[{}] Handshaking failed: {:?}.", self.peer.address, e);
                    }
                },
                Err(e) => {
                    warn!("[{}] Reading Handshake failed: {:?}.", self.peer.address, e);

                    Protocol::get().metrics.invalid_messages_inc();
                }
            }
        } else {
            warn!("[{}] Ignoring messages until fully handshaked.", self.peer.address);

            Protocol::get().metrics.invalid_messages_inc();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
