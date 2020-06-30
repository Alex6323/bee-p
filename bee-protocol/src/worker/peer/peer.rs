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
    worker::{MilestoneResponderWorkerEvent, TransactionResponderWorkerEvent, TransactionWorkerEvent},
};

use futures::{
    channel::{mpsc, oneshot},
    select,
    sink::SinkExt,
    stream::StreamExt,
};
use futures_util::{future, stream};
use log::{debug, error, info, warn};

use std::sync::Arc;

#[derive(Debug)]
pub(crate) enum PeerWorkerError {
    FailedSend,
}

enum PeerReadState {
    Header,
    Payload(Header),
}

struct PeerReadContext {
    state: PeerReadState,
    buffer: Vec<u8>,
}

pub struct PeerWorker {
    peer: Arc<HandshakedPeer>,
    transaction_worker: mpsc::Sender<TransactionWorkerEvent>,
    transaction_responder_worker: mpsc::Sender<TransactionResponderWorkerEvent>,
    milestone_responder_worker: mpsc::Sender<MilestoneResponderWorkerEvent>,
}

impl PeerWorker {
    pub fn new(peer: Arc<HandshakedPeer>) -> Self {
        Self {
            peer,
            transaction_worker: Protocol::get().transaction_worker.0.clone(),
            transaction_responder_worker: Protocol::get().transaction_responder_worker.0.clone(),
            milestone_responder_worker: Protocol::get().milestone_responder_worker.0.clone(),
        }
    }

    pub async fn run(
        mut self,
        mut receiver_fused: stream::Fuse<mpsc::Receiver<Vec<u8>>>,
        mut shutdown_fused: future::Fuse<oneshot::Receiver<()>>,
    ) {
        info!("[{}] Running.", self.peer.address);

        let mut context = PeerReadContext {
            state: PeerReadState::Header,
            buffer: Vec::new(),
        };

        loop {
            select! {
                event = receiver_fused.next() => {
                    if let Some(event) = event {
                        context = self.message_handler(context, event).await;
                    }
                },
                _ = shutdown_fused => {
                    break;
                }
            }
        }

        info!("[{}] Stopped.", self.peer.address);

        Protocol::get().peer_manager.remove(&self.peer.epid).await;
    }

    async fn process_message(&mut self, header: &Header, bytes: &[u8]) -> Result<(), PeerWorkerError> {
        match header.message_type {
            MilestoneRequest::ID => {
                debug!("[{}] Reading MilestoneRequest...", self.peer.address);
                match tlv_from_bytes::<MilestoneRequest>(&header, bytes) {
                    Ok(message) => {
                        self.milestone_responder_worker
                            .send(MilestoneResponderWorkerEvent {
                                epid: self.peer.epid,
                                request: message,
                            })
                            .await
                            .map_err(|_| PeerWorkerError::FailedSend)?;

                        self.peer.metrics.milestone_request_received_inc();
                        Protocol::get().metrics.milestone_request_received_inc();
                    }
                    Err(e) => {
                        warn!("[{}] Reading MilestoneRequest failed: {:?}.", self.peer.address, e);

                        self.peer.metrics.invalid_messages_received_inc();
                        Protocol::get().metrics.invalid_messages_received_inc();
                    }
                }
            }
            TransactionMessage::ID => {
                debug!("[{}] Reading TransactionMessage...", self.peer.address);
                match tlv_from_bytes::<TransactionMessage>(&header, bytes) {
                    Ok(message) => {
                        self.transaction_worker
                            .send(TransactionWorkerEvent {
                                from: self.peer.epid,
                                transaction: message,
                            })
                            .await
                            .map_err(|_| PeerWorkerError::FailedSend)?;

                        self.peer.metrics.transaction_received_inc();
                        Protocol::get().metrics.transaction_received_inc();
                    }
                    Err(e) => {
                        warn!("[{}] Reading TransactionMessage failed: {:?}.", self.peer.address, e);

                        self.peer.metrics.invalid_messages_received_inc();
                        Protocol::get().metrics.invalid_messages_received_inc();
                    }
                }
            }
            TransactionRequest::ID => {
                debug!("[{}] Reading TransactionRequest...", self.peer.address);
                match tlv_from_bytes::<TransactionRequest>(&header, bytes) {
                    Ok(message) => {
                        self.transaction_responder_worker
                            .send(TransactionResponderWorkerEvent {
                                epid: self.peer.epid,
                                request: message,
                            })
                            .await
                            .map_err(|_| PeerWorkerError::FailedSend)?;

                        self.peer.metrics.transaction_request_received_inc();
                        Protocol::get().metrics.transaction_request_received_inc();
                    }
                    Err(e) => {
                        warn!("[{}] Reading TransactionRequest failed: {:?}.", self.peer.address, e);

                        self.peer.metrics.invalid_messages_received_inc();
                        Protocol::get().metrics.invalid_messages_received_inc();
                    }
                }
            }
            Heartbeat::ID => {
                debug!("[{}] Reading Heartbeat...", self.peer.address);
                match tlv_from_bytes::<Heartbeat>(&header, bytes) {
                    Ok(message) => {
                        self.peer
                            .set_solid_milestone_index(message.solid_milestone_index.into());
                        self.peer
                            .set_snapshot_milestone_index(message.snapshot_milestone_index.into());

                        self.peer.metrics.heartbeat_received_inc();
                        Protocol::get().metrics.heartbeat_received_inc();
                    }
                    Err(e) => {
                        warn!("[{}] Reading Heartbeat failed: {:?}.", self.peer.address, e);

                        self.peer.metrics.invalid_messages_received_inc();
                        Protocol::get().metrics.invalid_messages_received_inc();
                    }
                }
            }
            _ => {
                warn!(
                    "[{}] Ignoring unsupported message type: {}.",
                    self.peer.address, header.message_type
                );

                self.peer.metrics.invalid_messages_received_inc();
                Protocol::get().metrics.invalid_messages_received_inc();
            }
        };

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
