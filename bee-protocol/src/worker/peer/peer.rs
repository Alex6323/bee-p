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
    peer::HandshakedPeer,
    protocol::Protocol,
    tangle::tangle,
    worker::{peer::MessageHandler, HasherWorkerEvent, MilestoneResponderWorkerEvent, TransactionResponderWorkerEvent},
};

use futures::channel::mpsc;
use log::{debug, error, info, warn};

use std::sync::Arc;

#[derive(Debug)]
pub(crate) enum PeerWorkerError {
    FailedSend,
}

pub struct PeerWorker {
    peer: Arc<HandshakedPeer>,
    hasher_worker: mpsc::UnboundedSender<HasherWorkerEvent>,
    transaction_responder_worker: mpsc::UnboundedSender<TransactionResponderWorkerEvent>,
    milestone_responder_worker: mpsc::UnboundedSender<MilestoneResponderWorkerEvent>,
}

impl PeerWorker {
    pub fn new(peer: Arc<HandshakedPeer>) -> Self {
        Self {
            peer,
            hasher_worker: Protocol::get().hasher_worker.clone(),
            transaction_responder_worker: Protocol::get().transaction_responder_worker.clone(),
            milestone_responder_worker: Protocol::get().milestone_responder_worker.clone(),
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
                debug!("[{}] Reading MilestoneRequest...", self.peer.address);
                match tlv_from_bytes::<MilestoneRequest>(&header, bytes) {
                    Ok(message) => {
                        self.milestone_responder_worker
                            .unbounded_send(MilestoneResponderWorkerEvent {
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
                debug!("[{}] Reading TransactionMessage...", self.peer.address);
                match tlv_from_bytes::<TransactionMessage>(&header, bytes) {
                    Ok(message) => {
                        self.hasher_worker
                            .unbounded_send(HasherWorkerEvent {
                                from: self.peer.epid,
                                transaction: message,
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
                debug!("[{}] Reading TransactionRequest...", self.peer.address);
                match tlv_from_bytes::<TransactionRequest>(&header, bytes) {
                    Ok(message) => {
                        self.transaction_responder_worker
                            .unbounded_send(TransactionResponderWorkerEvent {
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
                debug!("[{}] Reading Heartbeat...", self.peer.address);
                match tlv_from_bytes::<Heartbeat>(&header, bytes) {
                    Ok(message) => {
                        self.peer
                            .set_last_solid_milestone_index(message.last_solid_milestone_index.into());
                        self.peer
                            .set_snapshot_milestone_index(message.snapshot_milestone_index.into());
                        self.peer.set_last_milestone_index(message.last_milestone_index.into());
                        self.peer.set_connected_peers(message.connected_peers);
                        self.peer.set_synced_peers(message.synced_peers);

                        // // TODO Warn if can't help sync
                        if !tangle().is_synced() {
                            let index = *tangle().get_last_solid_milestone_index() + 1;

                            if !(index > message.snapshot_milestone_index
                                && index <= message.last_solid_milestone_index)
                            {
                                warn!("The peer {} can't help syncing.", self.peer.address);
                                // TODO Drop connection if autopeered.
                            }
                        }

                        // TODO think about a better solution
                        if Protocol::get().peer_manager.handshaked_peers.len() == 1 {
                            Protocol::request_milestone_fill();
                        }

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
