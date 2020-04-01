use crate::{
    message::{
        Handshake,
        Header,
        Heartbeat,
        LegacyGossip,
        Message,
        MilestoneRequest,
        TransactionBroadcast,
        TransactionRequest,
    },
    peer::{
        Peer,
        PeerMetrics,
    },
    protocol::{
        slice_eq,
        supported_version,
        Protocol,
        COORDINATOR_BYTES,
        MINIMUM_WEIGHT_MAGNITUDE,
        SUPPORTED_VERSIONS,
    },
    worker::{
        MilestoneResponderWorkerEvent,
        TransactionResponderWorkerEvent,
        TransactionWorkerEvent,
    },
};

use bee_network::{
    Command::SendBytes,
    Network,
    Origin,
};

use std::{
    sync::Arc,
    time::{
        SystemTime,
        UNIX_EPOCH,
    },
};

use futures::{
    channel::{
        mpsc,
        oneshot,
    },
    future::FutureExt,
    select,
    sink::SinkExt,
    stream::StreamExt,
};
use log::*;

#[derive(Debug)]
pub(crate) enum ReceiverWorkerError {
    FailedSend,
}

enum ReceiverWorkerMessageState {
    Header,
    Payload(Header),
}

struct AwaitingHandshakeContext {
    state: ReceiverWorkerMessageState,
}

struct AwaitingMessageContext {
    state: ReceiverWorkerMessageState,
    buffer: Vec<u8>,
}

enum ReceiverWorkerState {
    AwaitingHandshake(AwaitingHandshakeContext),
    AwaitingMessage(AwaitingMessageContext),
}

pub struct ReceiverWorker {
    network: Network,
    peer: Arc<Peer>,
    metrics: Arc<PeerMetrics>,
    transaction_worker: mpsc::Sender<TransactionWorkerEvent>,
    transaction_responder_worker: mpsc::Sender<TransactionResponderWorkerEvent>,
    milestone_responder_worker: mpsc::Sender<MilestoneResponderWorkerEvent>,
}

impl ReceiverWorker {
    pub fn new(network: Network, peer: Arc<Peer>, metrics: Arc<PeerMetrics>) -> Self {
        Self {
            network,
            peer,
            metrics,
            transaction_worker: Protocol::get().transaction_worker.0.clone(),
            transaction_responder_worker: Protocol::get().transaction_responder_worker.0.clone(),
            milestone_responder_worker: Protocol::get().milestone_responder_worker.0.clone(),
        }
    }

    pub async fn run(mut self, bytes_receiver: mpsc::Receiver<Vec<u8>>, shutdown_receiver: oneshot::Receiver<()>) {
        info!("[Peer({})] Receiver worker running.", self.peer.epid);

        let mut state = ReceiverWorkerState::AwaitingHandshake(AwaitingHandshakeContext {
            state: ReceiverWorkerMessageState::Header,
        });
        let mut bytes_fused = bytes_receiver.fuse();
        let mut shutdown_fused = shutdown_receiver.fuse();

        // This is the only message not using a SenderWorker because they are not running yet (awaiting handshake)
        if let Err(e) = self
            .network
            .send(SendBytes {
                epid: self.peer.epid,
                // TODO port
                bytes: Handshake::new(1337, &COORDINATOR_BYTES, MINIMUM_WEIGHT_MAGNITUDE, &SUPPORTED_VERSIONS)
                    .into_full_bytes(),
                responder: None,
            })
            .await
        {
            // TODO then what ?
            warn!("[Peer({})] Failed to send handshake: {:?}.", self.peer.epid, e);
        }

        loop {
            select! {
                event = bytes_fused.next() => {
                    if let Some(event) = event {
                        state = match state {
                            ReceiverWorkerState::AwaitingHandshake(context) => self.handshake_handler(context, event).await,
                            ReceiverWorkerState::AwaitingMessage(context) => self.message_handler(context, event).await,
                        }
                    }
                },
                _ = shutdown_fused => {
                    break;
                }
            }
        }

        info!("[Peer({})] Receiver worker shut down.", self.peer.epid);

        Protocol::senders_remove(&self.peer.epid).await;
    }

    async fn check_handshake(&self, header: Header, bytes: &[u8]) -> ReceiverWorkerState {
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

                if ((timestamp - handshake.timestamp) as i64).abs() > 5000 {
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
                    match self.peer.origin {
                        Origin::Outbound => {
                            // TODO use Port instead or deref
                            if handshake.port != *self.peer.address.port() {
                                warn!(
                                    "[Peer({})] Invalid handshake port: {} != {}.",
                                    self.peer.epid, handshake.port, handshake.port
                                );
                            }
                        }
                        Origin::Inbound => {
                            // TODO check if whitelisted
                        }
                        Origin::Unbound => {
                            error!("[Peer({})] Unbound peer origin.", self.peer.epid);
                        }
                    }

                    info!("[Peer({})] Handshake completed.", self.peer.epid);

                    Protocol::senders_add(self.network.clone(), self.peer.clone(), self.metrics.clone()).await;

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

    async fn handshake_handler(&mut self, context: AwaitingHandshakeContext, bytes: Vec<u8>) -> ReceiverWorkerState {
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
                        self.check_handshake(header, &bytes[3..bytes.len()]).await
                    } else {
                        ReceiverWorkerState::AwaitingHandshake(AwaitingHandshakeContext {
                            state: ReceiverWorkerMessageState::Payload(header),
                        })
                    }
                }
                ReceiverWorkerMessageState::Payload(header) => {
                    self.check_handshake(header, &bytes[..bytes.len()]).await
                }
            }
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
                        self.milestone_responder_worker
                            .send(MilestoneResponderWorkerEvent {
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
                        self.transaction_worker
                            .send(message)
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
                        self.transaction_responder_worker
                            .send(TransactionResponderWorkerEvent {
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
        mut bytes: Vec<u8>,
    ) -> ReceiverWorkerState {
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
                    if offset + 3 <= context.buffer.len() {
                        debug!("[Peer({})] Reading Header...", self.peer.epid);
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
}

#[cfg(test)]
mod tests {

    use super::*;
}
