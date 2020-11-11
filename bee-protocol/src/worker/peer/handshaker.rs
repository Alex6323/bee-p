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
    config::ProtocolConfig,
    event::HandshakeCompleted,
    packet::{tlv_from_bytes, tlv_into_bytes, Header, Packet, MINIMUM_VERSION},
    peer::Peer,
    protocol::Protocol,
    tangle::MsTangle,
    worker::{
        peer::message_handler::MessageHandler, HasherWorkerEvent, MessageResponderWorkerEvent,
        MilestoneRequesterWorkerEvent, MilestoneResponderWorkerEvent, PeerWorker, RequestedMilestones,
    },
};

use bee_common_ext::node::ResHandle;
use bee_network::{Command::SendMessage, Network, Origin};
use bee_storage::storage::Backend;

use futures::{channel::oneshot, future::FutureExt};
use log::{error, info, trace, warn};
use tokio::spawn;

use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug)]
pub(crate) enum HandshakeError {
    InvalidTimestampDiff(i64),
    CoordinatorMismatch,
    MwmMismatch(u8, u8),
    UnsupportedVersion(u16),
    PortMismatch(u16, u16),
    AlreadyHandshaked,
}

#[derive(Debug)]
pub(crate) enum PeerHandshakerWorkerError {}

enum HandshakeStatus {
    Awaiting,
    Done,
}

pub struct PeerHandshakerWorker {
    network: Network,
    config: ProtocolConfig,
    peer: Arc<Peer>,
    status: HandshakeStatus,
    hasher: flume::Sender<HasherWorkerEvent>,
    message_responder: flume::Sender<MessageResponderWorkerEvent>,
    milestone_responder: flume::Sender<MilestoneResponderWorkerEvent>,
    milestone_requester: flume::Sender<MilestoneRequesterWorkerEvent>,
}

impl PeerHandshakerWorker {
    pub(crate) fn new(
        network: Network,
        config: ProtocolConfig,
        peer: Arc<Peer>,
        hasher: flume::Sender<HasherWorkerEvent>,
        message_responder: flume::Sender<MessageResponderWorkerEvent>,
        milestone_responder: flume::Sender<MilestoneResponderWorkerEvent>,
        milestone_requester: flume::Sender<MilestoneRequesterWorkerEvent>,
    ) -> Self {
        Self {
            network,
            config,
            peer,
            status: HandshakeStatus::Awaiting,
            hasher,
            message_responder,
            milestone_responder,
            milestone_requester,
        }
    }

    pub async fn run<B: Backend>(
        mut self,
        tangle: ResHandle<MsTangle<B>>,
        requested_milestones: ResHandle<RequestedMilestones>,
        receiver: flume::Receiver<Vec<u8>>,
        shutdown: oneshot::Receiver<()>,
    ) {
        info!("[{}] Running.", self.peer.address);

        // TODO should we have a first check if already connected ?

        let receiver_fused = receiver.into_stream();
        let shutdown_fused = shutdown.fuse();

        // // This is the only message not using a Sender because they are not running yet (awaiting handshake)
        // if let Err(e) = self.network.unbounded_send(SendMessage {
        //     receiver_epid: self.peer.epid,
        //     message: tlv_into_bytes(Handshake::new(
        //         self.network.config().binding_port,
        //         &self.config.coordinator.public_key,
        //         self.config.mwm,
        //         MINIMUM_VERSION,
        //     )),
        // }) {
        //     // TODO then what ?
        //     warn!("[{}] Failed to send handshake: {:?}.", self.peer.address, e);
        // }

        let mut message_handler = MessageHandler::new(receiver_fused, shutdown_fused, self.peer.address.clone());

        while let Some((header, bytes)) = message_handler.fetch_message().await {
            if let Err(e) = self
                .process_message(&tangle, &requested_milestones, &header, bytes)
                .await
            {
                error!("[{}] Processing message failed: {:?}.", self.peer.address, e);
            }
            if let HandshakeStatus::Done = self.status {
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
                            .get(&self.peer.id)
                            .unwrap()
                            .value()
                            .clone(),
                        self.hasher,
                        self.message_responder,
                        self.milestone_responder,
                    )
                    .run(tangle.clone(), message_handler),
                );
            }
            _ => (),
        }

        info!("[{}] Stopped.", self.peer.address);
    }

    // pub(crate) fn validate_handshake(&mut self, handshake: Handshake) -> Result<SocketAddr, HandshakeError> {
    //     let timestamp = SystemTime::now()
    //         .duration_since(UNIX_EPOCH)
    //         .expect("Clock may have gone backwards")
    //         .as_millis() as u64;
    //
    //     if ((timestamp - handshake.timestamp) as i64).abs() as u64 > self.config.handshake_window * 1000 {
    //         return Err(HandshakeError::InvalidTimestampDiff(
    //             ((timestamp - handshake.timestamp) as i64).abs(),
    //         ));
    //     }
    //
    //     if !self.config.coordinator.public_key.eq(&handshake.coordinator) {
    //         return Err(HandshakeError::CoordinatorMismatch);
    //     }
    //
    //     if self.config.mwm != handshake.minimum_weight_magnitude {
    //         return Err(HandshakeError::MwmMismatch(
    //             self.config.mwm,
    //             handshake.minimum_weight_magnitude,
    //         ));
    //     }
    //
    //     if handshake.version < MINIMUM_VERSION {
    //         return Err(HandshakeError::UnsupportedVersion(handshake.version));
    //     }
    //
    //     let address = match self.peer.origin {
    //         Origin::Outbound => {
    //             if self.peer.address.port() != handshake.port {
    //                 return Err(HandshakeError::PortMismatch(self.peer.address.port(), handshake.port));
    //             }
    //
    //             self.peer.address
    //         }
    //         Origin::Inbound => {
    //             // TODO check if whitelisted
    //
    //             SocketAddr::new(self.peer.address.ip(), handshake.port)
    //         }
    //     };
    //
    //     for peer in Protocol::get().peer_manager.handshaked_peers.iter() {
    //         if peer.address == address {
    //             self.status = HandshakeStatus::Duplicate;
    //             return Err(HandshakeError::AlreadyHandshaked);
    //         }
    //     }
    //
    //     Ok(address)
    // }

    async fn process_message<B: Backend>(
        &mut self,
        tangle: &MsTangle<B>,
        requested_milestones: &RequestedMilestones,
        header: &Header,
        bytes: &[u8],
    ) -> Result<(), PeerHandshakerWorkerError> {
        // if let Handshake::ID = header.packet_type {
        //     trace!("[{}] Reading Handshake...", self.peer.address);
        //     match tlv_from_bytes::<Handshake>(&header, bytes) {
        //         Ok(handshake) => match self.validate_handshake(handshake) {
        //             Ok(address) => {
        //                 info!("[{}] Handshake completed.", self.peer.address);
        //
        //                 Protocol::get().peer_manager.handshake(&self.peer.epid, address).await;
        //
        //                 Protocol::get()
        //                     .bus
        //                     .dispatch(HandshakeCompleted(self.peer.epid, address));
        //
        //                 Protocol::send_heartbeat(
        //                     self.peer.epid,
        //                     tangle.get_latest_solid_milestone_index(),
        //                     tangle.get_pruning_index(),
        //                     tangle.get_latest_milestone_index(),
        //                 );
        //
        //                 Protocol::request_latest_milestone(tangle, &self.milestone_requester,
        // &*requested_milestones,Some(self.peer.epid));
        //
        //                 self.status = HandshakeStatus::Done;
        //             }
        //             Err(e) => {
        //                 warn!("[{}] Handshaking failed: {:?}.", self.peer.address, e);
        //             }
        //         },
        //         Err(e) => {
        //             warn!("[{}] Reading Handshake failed: {:?}.", self.peer.address, e);
        //
        //             Protocol::get().metrics.invalid_messages_inc();
        //         }
        //     }
        // } else {
        //     warn!("[{}] Ignoring messages until fully handshaked.", self.peer.address);
        //
        //     Protocol::get().metrics.invalid_messages_inc();
        // }

        Ok(())
    }
}
