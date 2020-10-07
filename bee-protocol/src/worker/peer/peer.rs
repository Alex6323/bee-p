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
    message::{
        tlv_from_bytes, Header, Heartbeat, Message, MilestoneRequest, Transaction as TransactionMessage,
        TransactionRequest,
    },
    milestone::MilestoneIndex,
    peer::HandshakedPeer,
    protocol::Protocol,
    tangle::tangle,
    worker::{
        peer::message_handler::MessageHandler, HasherWorkerEvent, MilestoneResponderWorkerEvent,
        TransactionResponderWorkerEvent,
    },
};

use log::{error, info, trace, warn};

use std::sync::Arc;

#[derive(Debug)]
pub(crate) enum PeerWorkerError {
    FailedSend,
}

pub struct PeerWorker {
    peer: Arc<HandshakedPeer>,
    hasher: flume::Sender<HasherWorkerEvent>,
    transaction_responder: flume::Sender<TransactionResponderWorkerEvent>,
    milestone_responder: flume::Sender<MilestoneResponderWorkerEvent>,
}

impl PeerWorker {
    pub(crate) fn new(
        peer: Arc<HandshakedPeer>,
        hasher: flume::Sender<HasherWorkerEvent>,
        transaction_responder: flume::Sender<TransactionResponderWorkerEvent>,
        milestone_responder: flume::Sender<MilestoneResponderWorkerEvent>,
    ) -> Self {
        Self {
            peer,
            hasher,
            transaction_responder,
            milestone_responder,
        }
    }

    pub(super) async fn run(mut self, mut message_handler: MessageHandler) {
        info!("[{}] Running.", self.peer.address);

        while let Some((header, bytes)) = message_handler.fetch_message().await {
            if let Err(e) = self.process_message(&header, bytes) {
                error!("[{}] Processing message failed: {:?}.", self.peer.address, e);
            }
        }

        info!("[{}] Stopped.", self.peer.address);

        Protocol::get().peer_manager.remove(&self.peer.epid).await;
    }

    fn process_message(&mut self, header: &Header, bytes: &[u8]) -> Result<(), PeerWorkerError> {
        match header.message_type {
            MilestoneRequest::ID => {
                trace!("[{}] Reading MilestoneRequest...", self.peer.address);
                match tlv_from_bytes::<MilestoneRequest>(&header, bytes) {
                    Ok(message) => {
                        self.milestone_responder
                            .send(MilestoneResponderWorkerEvent {
                                epid: self.peer.epid,
                                request: message,
                            })
                            .map_err(|_| PeerWorkerError::FailedSend)?;

                        self.peer.metrics.milestone_requests_received_inc();
                        Protocol::get().metrics.milestone_requests_received_inc();
                    }
                    Err(e) => {
                        warn!("[{}] Reading MilestoneRequest failed: {:?}.", self.peer.address, e);

                        self.peer.metrics.invalid_messages_inc();
                        Protocol::get().metrics.invalid_messages_inc();
                    }
                }
            }
            TransactionMessage::ID => {
                trace!("[{}] Reading TransactionMessage...", self.peer.address);
                match tlv_from_bytes::<TransactionMessage>(&header, bytes) {
                    Ok(message) => {
                        self.hasher
                            .send(HasherWorkerEvent {
                                from: self.peer.epid,
                                transaction_message: message,
                            })
                            .map_err(|_| PeerWorkerError::FailedSend)?;

                        self.peer.metrics.transactions_received_inc();
                        Protocol::get().metrics.transactions_received_inc();
                    }
                    Err(e) => {
                        warn!("[{}] Reading TransactionMessage failed: {:?}.", self.peer.address, e);

                        self.peer.metrics.invalid_messages_inc();
                        Protocol::get().metrics.invalid_messages_inc();
                    }
                }
            }
            TransactionRequest::ID => {
                trace!("[{}] Reading TransactionRequest...", self.peer.address);
                match tlv_from_bytes::<TransactionRequest>(&header, bytes) {
                    Ok(message) => {
                        self.transaction_responder
                            .send(TransactionResponderWorkerEvent {
                                epid: self.peer.epid,
                                request: message,
                            })
                            .map_err(|_| PeerWorkerError::FailedSend)?;

                        self.peer.metrics.transaction_requests_received_inc();
                        Protocol::get().metrics.transaction_requests_received_inc();
                    }
                    Err(e) => {
                        warn!("[{}] Reading TransactionRequest failed: {:?}.", self.peer.address, e);

                        self.peer.metrics.invalid_messages_inc();
                        Protocol::get().metrics.invalid_messages_inc();
                    }
                }
            }
            Heartbeat::ID => {
                trace!("[{}] Reading Heartbeat...", self.peer.address);
                match tlv_from_bytes::<Heartbeat>(&header, bytes) {
                    Ok(message) => {
                        self.peer
                            .set_latest_solid_milestone_index(message.latest_solid_milestone_index.into());
                        self.peer.set_pruned_index(message.pruned_index.into());
                        self.peer
                            .set_latest_milestone_index(message.latest_milestone_index.into());
                        self.peer.set_connected_peers(message.connected_peers);
                        self.peer.set_synced_peers(message.synced_peers);

                        if !tangle().is_synced_threshold(2)
                            && !self
                                .peer
                                .has_data(MilestoneIndex(*tangle().get_latest_solid_milestone_index() + 1))
                        {
                            warn!("The peer {} can't help syncing.", self.peer.address);
                            // TODO drop if autopeered.
                        }

                        // Also drop connection if autopeered and we can't help it sync

                        self.peer.metrics.heartbeats_received_inc();
                        Protocol::get().metrics.heartbeats_received_inc();
                    }
                    Err(e) => {
                        warn!("[{}] Reading Heartbeat failed: {:?}.", self.peer.address, e);

                        self.peer.metrics.invalid_messages_inc();
                        Protocol::get().metrics.invalid_messages_inc();
                    }
                }
            }
            _ => {
                warn!(
                    "[{}] Ignoring unsupported message type: {}.",
                    self.peer.address, header.message_type
                );

                self.peer.metrics.invalid_messages_inc();
                Protocol::get().metrics.invalid_messages_inc();
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
