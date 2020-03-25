use crate::message::{
    Handshake,
    Header,
    Heartbeat,
    LegacyGossip,
    Message,
    MilestoneRequest,
    TransactionBroadcast,
    TransactionRequest,
};
use crate::peer::{
    Peer,
    PeerMetrics,
};
use crate::protocol::{
    slice_eq,
    supported_version,
    COORDINATOR_BYTES,
    MINIMUM_WEIGHT_MAGNITUDE,
    SUPPORTED_VERSIONS,
};
use crate::worker::{
    sender_registry,
    ResponderWorkerEvent,
    SenderWorkerEvent,
    TransactionWorkerEvent,
};

use bee_network::EndpointId;

use std::sync::Arc;
use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

use futures::channel::mpsc::{
    Receiver,
    Sender,
};
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use log::*;

#[derive(Debug)]
pub(crate) enum ReceiverWorkerError {
    FailedSend,
}

pub enum ReceiverWorkerEvent {
    Removed,
    Connected,
    Disconnected,
    Message(Vec<u8>),
}

enum ReceiverWorkerMessageState {
    Header,
    Payload(Header),
}

struct AwaitingConnectionContext {}

struct AwaitingHandshakeContext {
    state: ReceiverWorkerMessageState,
}

struct AwaitingMessageContext {
    state: ReceiverWorkerMessageState,
    buffer: Vec<u8>,
}

enum ReceiverWorkerState {
    AwaitingConnection(AwaitingConnectionContext),
    AwaitingHandshake(AwaitingHandshakeContext),
    AwaitingMessage(AwaitingMessageContext),
}

pub struct ReceiverWorker {
    peer: Arc<Peer>,
    metrics: Arc<PeerMetrics>,
    receiver: Receiver<ReceiverWorkerEvent>,
    transaction_worker_sender: Sender<TransactionWorkerEvent>,
    responder_worker: Sender<ResponderWorkerEvent>,
}

impl ReceiverWorker {
    pub fn new(
        peer: Arc<Peer>,
        metrics: Arc<PeerMetrics>,
        receiver: Receiver<ReceiverWorkerEvent>,
        transaction_worker_sender: Sender<TransactionWorkerEvent>,
        responder_worker: Sender<ResponderWorkerEvent>,
    ) -> Self {
        Self {
            peer,
            metrics,
            receiver,
            transaction_worker_sender,
            responder_worker,
        }
    }

    async fn send_handshake(&mut self) {
        // TODO port
        let handshake = Handshake::new(1337, &COORDINATOR_BYTES, MINIMUM_WEIGHT_MAGNITUDE, &SUPPORTED_VERSIONS);

        if let Some(context) = sender_registry().contexts().read().await.get(&self.peer.epid) {
            if let Err(e) = context
                .handshake_sender
                // TODO avoid clone
                .clone()
                .send(SenderWorkerEvent::Message(handshake))
                .await
            {
                warn!("[ResponderWorker ] Sending message failed: {}.", e);
            }
        };
    }

    pub async fn run(mut self) {
        info!("[Peer({})] Receiver worker running.", self.peer.epid);

        let mut state = ReceiverWorkerState::AwaitingConnection(AwaitingConnectionContext {});

        while let Some(event) = self.receiver.next().await {
            if let ReceiverWorkerEvent::Removed = event {
                info!("[Peer({})] Receiver worker shut down.", self.peer.epid);
                break;
            }

            state = match state {
                ReceiverWorkerState::AwaitingConnection(context) => self.connection_handler(context, event).await,
                ReceiverWorkerState::AwaitingHandshake(context) => self.handshake_handler(context, event).await,
                ReceiverWorkerState::AwaitingMessage(context) => self.message_handler(context, event).await,
            }
        }
    }

    async fn connection_handler(
        &mut self,
        context: AwaitingConnectionContext,
        event: ReceiverWorkerEvent,
    ) -> ReceiverWorkerState {
        match event {
            ReceiverWorkerEvent::Connected => {
                info!("[Peer({})] Connected.", self.peer.epid);

                // TODO spawn ?
                self.send_handshake().await;

                ReceiverWorkerState::AwaitingHandshake(AwaitingHandshakeContext {
                    state: ReceiverWorkerMessageState::Header,
                })
            }
            _ => ReceiverWorkerState::AwaitingConnection(context),
        }
    }

    fn check_handshake(&self, header: Header, bytes: &[u8]) -> ReceiverWorkerState {
        debug!("[Peer({})] Reading Handshake...", self.peer.epid);

        match Handshake::from_full_bytes(&header, bytes) {
            Ok(handshake) => {
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Clock may have gone backwards")
                    .as_millis() as u64;
                let mut state = ReceiverWorkerState::AwaitingHandshake(AwaitingHandshakeContext {
                    state: ReceiverWorkerMessageState::Header,
                });

                // TODO check actual port
                if handshake.port != handshake.port {
                    warn!(
                        "[Peer({})] Invalid handshake port: {} != {}.",
                        self.peer.epid, handshake.port, handshake.port
                    );
                } else if ((timestamp - handshake.timestamp) as i64).abs() > 5000 {
                    warn!(
                        "[Peer({})] Invalid handshake timestamp, difference of {}ms.",
                        self.peer.epid,
                        ((timestamp - handshake.timestamp) as i64).abs()
                    );
                } else if !slice_eq(&handshake.coordinator, &COORDINATOR_BYTES) {
                    warn!("[Peer({})] Invalid handshake coordinator.", self.peer.epid);
                } else if handshake.minimum_weight_magnitude != MINIMUM_WEIGHT_MAGNITUDE {
                    warn!(
                        "[Peer({})] Invalid handshake MWM: {} != {}.",
                        self.peer.epid, handshake.minimum_weight_magnitude, MINIMUM_WEIGHT_MAGNITUDE
                    );
                } else if let Err(version) = supported_version(&handshake.supported_messages) {
                    warn!("[Peer({})] Unsupported protocol version: {}.", self.peer.epid, version);
                } else {
                    // TODO check duplicate connection
                    info!("[Peer({})] Handshake completed.", self.peer.epid);

                    state = ReceiverWorkerState::AwaitingMessage(AwaitingMessageContext {
                        state: ReceiverWorkerMessageState::Header,
                        buffer: Vec::new(),
                    });
                }

                state
            }

            Err(e) => {
                warn!("[Peer({})] Reading Handshake failed: {:?}.", self.peer.epid, e);

                ReceiverWorkerState::AwaitingHandshake(AwaitingHandshakeContext {
                    state: ReceiverWorkerMessageState::Header,
                })
            }
        }
    }

    async fn handshake_handler(
        &mut self,
        context: AwaitingHandshakeContext,
        event: ReceiverWorkerEvent,
    ) -> ReceiverWorkerState {
        match event {
            ReceiverWorkerEvent::Disconnected => {
                info!("[Peer({})] Disconnected.", self.peer.epid);

                ReceiverWorkerState::AwaitingConnection(AwaitingConnectionContext {})
            }
            ReceiverWorkerEvent::Message(bytes) => {
                // TODO needed ?
                if bytes.len() < 3 {
                    ReceiverWorkerState::AwaitingHandshake(AwaitingHandshakeContext {
                        state: ReceiverWorkerMessageState::Header,
                    })
                } else {
                    match context.state {
                        ReceiverWorkerMessageState::Header => {
                            debug!("[Peer({})] Reading Header...", self.peer.epid);

                            let header = Header::from_bytes(&bytes[0..3]);

                            if bytes.len() > 3 {
                                self.check_handshake(header, &bytes[3..bytes.len()])
                            } else {
                                ReceiverWorkerState::AwaitingHandshake(AwaitingHandshakeContext {
                                    state: ReceiverWorkerMessageState::Payload(header),
                                })
                            }
                        }
                        ReceiverWorkerMessageState::Payload(header) => {
                            self.check_handshake(header, &bytes[..bytes.len()])
                        }
                    }
                }
            }
            _ => ReceiverWorkerState::AwaitingHandshake(context),
        }
    }

    async fn process_message(&mut self, header: &Header, bytes: &[u8]) -> Result<(), ReceiverWorkerError> {
        // TODO metrics
        match header.message_type {
            Handshake::ID => {
                warn!("[Peer({})] Ignoring unexpected Handshake.", self.peer.epid);

                self.peer.metrics.handshake_received_inc();
                self.metrics.handshake_received_inc();
                // TODO handle here instead of dedicated state ?
            }

            LegacyGossip::ID => {
                warn!("[Peer({})] Ignoring unsupported LegacyGossip.", self.peer.epid);
            }

            MilestoneRequest::ID => {
                debug!("[Peer({})] Reading MilestoneRequest...", self.peer.epid);

                self.peer.metrics.milestone_request_received_inc();
                self.metrics.milestone_request_received_inc();

                match MilestoneRequest::from_full_bytes(&header, bytes) {
                    Ok(message) => {
                        self.responder_worker
                            .send(ResponderWorkerEvent::MilestoneRequest {
                                epid: self.peer.epid,
                                message: message,
                            })
                            .await
                            .map_err(|_| ReceiverWorkerError::FailedSend)?;
                    }
                    Err(e) => {
                        warn!("[Peer({})] Reading MilestoneRequest failed: {:?}.", self.peer.epid, e);
                    }
                }
            }

            TransactionBroadcast::ID => {
                debug!("[Peer({})] Reading TransactionBroadcast...", self.peer.epid);

                self.peer.metrics.transaction_broadcast_received_inc();
                self.metrics.transaction_broadcast_received_inc();

                match TransactionBroadcast::from_full_bytes(&header, bytes) {
                    Ok(message) => {
                        self.transaction_worker_sender
                            .send(TransactionWorkerEvent::Transaction(message))
                            .await
                            .map_err(|_| ReceiverWorkerError::FailedSend)?;
                    }
                    Err(e) => {
                        warn!(
                            "[Peer({})] Reading TransactionBroadcast failed: {:?}.",
                            self.peer.epid, e
                        );
                    }
                }
            }

            TransactionRequest::ID => {
                debug!("[Peer({})] Reading TransactionRequest...", self.peer.epid);

                self.peer.metrics.transaction_request_received_inc();
                self.metrics.transaction_request_received_inc();

                match TransactionRequest::from_full_bytes(&header, bytes) {
                    Ok(message) => {
                        self.responder_worker
                            .send(ResponderWorkerEvent::TransactionRequest {
                                epid: self.peer.epid,
                                message: message,
                            })
                            .await
                            .map_err(|_| ReceiverWorkerError::FailedSend)?;
                    }
                    Err(e) => {
                        warn!("[Peer({})] Reading TransactionRequest failed: {:?}.", self.peer.epid, e);
                    }
                }
            }

            Heartbeat::ID => {
                debug!("[Peer({})] Reading Heartbeat...", self.peer.epid);

                self.peer.metrics.heartbeat_received_inc();
                self.metrics.heartbeat_received_inc();

                match Heartbeat::from_full_bytes(&header, bytes) {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("[Peer({})] Reading Heartbeat failed: {:?}.", self.peer.epid, e);
                    }
                }
            }

            _ => {
                // _ => Err(MessageError::InvalidMessageType(message_type)),
            }
        };

        Ok(())
    }

    async fn message_handler(
        &mut self,
        mut context: AwaitingMessageContext,
        event: ReceiverWorkerEvent,
    ) -> ReceiverWorkerState {
        match event {
            ReceiverWorkerEvent::Disconnected => {
                debug!("[Peer({})] Disconnected.", self.peer.epid);

                ReceiverWorkerState::AwaitingConnection(AwaitingConnectionContext {})
            }
            ReceiverWorkerEvent::Message(mut bytes) => {
                let mut offset = 0;
                let mut remaining = true;

                if context.buffer.is_empty() {
                    context.buffer = bytes;
                } else {
                    context.buffer.append(&mut bytes);
                }

                while remaining {
                    context.state = match context.state {
                        ReceiverWorkerMessageState::Header => {
                            debug!("[Peer({})] Reading Header...", self.peer.epid);

                            if offset + 3 <= context.buffer.len() {
                                let header = Header::from_bytes(&context.buffer[offset..offset + 3]);
                                offset = offset + 3;

                                ReceiverWorkerMessageState::Payload(header)
                            } else {
                                remaining = false;

                                ReceiverWorkerMessageState::Header
                            }
                        }
                        ReceiverWorkerMessageState::Payload(header) => {
                            if (offset + header.message_length as usize) <= context.buffer.len() {
                                if let Err(e) = self
                                    .process_message(
                                        &header,
                                        &context.buffer[offset..offset + header.message_length as usize],
                                    )
                                    .await
                                {
                                    error!("[Peer({})] Processing message failed: {:?}.", self.peer.epid, e);
                                }

                                offset = offset + header.message_length as usize;

                                ReceiverWorkerMessageState::Header
                            } else {
                                remaining = false;

                                ReceiverWorkerMessageState::Payload(header)
                            }
                        }
                    };
                }

                ReceiverWorkerState::AwaitingMessage(AwaitingMessageContext {
                    state: context.state,
                    buffer: context.buffer[offset..].to_vec(),
                })
            }
            _ => ReceiverWorkerState::AwaitingMessage(context),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
}
