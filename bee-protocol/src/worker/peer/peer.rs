use crate::{
    message::{
        Handshake,
        Header,
        Heartbeat,
        Message,
        MilestoneRequest,
        TransactionBroadcast,
        TransactionRequest,
    },
    peer::Peer,
    protocol::{
        Protocol,
        SUPPORTED_VERSIONS,
    },
    worker::{
        peer::validate_handshake,
        MilestoneResponderWorkerEvent,
        TransactionResponderWorkerEvent,
        TransactionWorkerEvent,
    },
};

use bee_network::{
    Command::SendMessage,
    Network,
};
use bee_tangle::tangle;

use std::sync::Arc;

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
    network: Network,
    peer: Arc<Peer>,
    transaction_worker: mpsc::Sender<TransactionWorkerEvent>,
    transaction_responder_worker: mpsc::Sender<TransactionResponderWorkerEvent>,
    milestone_responder_worker: mpsc::Sender<MilestoneResponderWorkerEvent>,
    handshaked: bool,
}

impl PeerWorker {
    pub fn new(network: Network, peer: Arc<Peer>) -> Self {
        Self {
            network,
            peer,
            transaction_worker: Protocol::get().transaction_worker.0.clone(),
            transaction_responder_worker: Protocol::get().transaction_responder_worker.0.clone(),
            milestone_responder_worker: Protocol::get().milestone_responder_worker.0.clone(),
            handshaked: false,
        }
    }

    pub async fn run(mut self, receiver: mpsc::Receiver<Vec<u8>>, shutdown: oneshot::Receiver<()>) {
        info!("[PeerWorker({})] Running.", self.peer.epid);

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
                // TODO port
                bytes: Handshake::new(
                    1337,
                    &Protocol::get().conf.coordinator.public_key_bytes,
                    Protocol::get().conf.mwm,
                    &SUPPORTED_VERSIONS,
                )
                .into_full_bytes(),
                responder: None,
            })
            .await
        {
            // TODO then what ?
            warn!("[PeerWorker({})] Failed to send handshake: {:?}.", self.peer.epid, e);
        }

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

        info!("[PeerWorker({})] Stopped.", self.peer.epid);

        Protocol::senders_remove(&self.peer.epid).await;
    }

    async fn process_message(&mut self, header: &Header, bytes: &[u8]) -> Result<(), PeerWorkerError> {
        match self.handshaked {
            false => match header.message_type {
                Handshake::ID => {
                    debug!("[PeerWorker({})] Reading Handshake...", self.peer.epid);
                    match Handshake::from_full_bytes(&header, bytes) {
                        Ok(handshake) => match validate_handshake(&self.peer, handshake) {
                            Ok(_) => {
                                info!("[PeerWorker({})] Handshake completed.", self.peer.epid);

                                Protocol::senders_add(self.network.clone(), self.peer.clone()).await;

                                Protocol::send_heartbeat(
                                    self.peer.epid,
                                    *tangle().get_solid_milestone_index(),
                                    *tangle().get_snapshot_milestone_index(),
                                )
                                .await;

                                Protocol::request_last_milestone(Some(self.peer.epid));
                                Protocol::trigger_milestone_solidification().await;

                                self.handshaked = true;
                            }
                            Err(e) => {}
                        },
                        Err(e) => {
                            warn!("[PeerWorker({})] Reading Handshake failed: {:?}.", self.peer.epid, e);

                            self.peer.metrics.invalid_messages_received_inc();
                            Protocol::get().metrics.invalid_messages_received_inc();
                        }
                    }
                }
                _ => {
                    warn!(
                        "[PeerWorker({})] Ignoring messages until fully handshaked.",
                        self.peer.epid
                    );

                    self.peer.metrics.invalid_messages_received_inc();
                    Protocol::get().metrics.invalid_messages_received_inc();
                }
            },
            true => {
                match header.message_type {
                    MilestoneRequest::ID => {
                        debug!("[PeerWorker({})] Reading MilestoneRequest...", self.peer.epid);
                        match MilestoneRequest::from_full_bytes(&header, bytes) {
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
                                warn!(
                                    "[PeerWorker({})] Reading MilestoneRequest failed: {:?}.",
                                    self.peer.epid, e
                                );

                                self.peer.metrics.invalid_messages_received_inc();
                                Protocol::get().metrics.invalid_messages_received_inc();
                            }
                        }
                    }
                    TransactionBroadcast::ID => {
                        debug!("[PeerWorker({})] Reading TransactionBroadcast...", self.peer.epid);
                        match TransactionBroadcast::from_full_bytes(&header, bytes) {
                            Ok(message) => {
                                self.transaction_worker
                                    .send(TransactionWorkerEvent {
                                        from: self.peer.epid,
                                        transaction_broadcast: message,
                                    })
                                    .await
                                    .map_err(|_| PeerWorkerError::FailedSend)?;

                                self.peer.metrics.transaction_broadcast_received_inc();
                                Protocol::get().metrics.transaction_broadcast_received_inc();
                            }
                            Err(e) => {
                                warn!(
                                    "[PeerWorker({})] Reading TransactionBroadcast failed: {:?}.",
                                    self.peer.epid, e
                                );

                                self.peer.metrics.invalid_messages_received_inc();
                                Protocol::get().metrics.invalid_messages_received_inc();
                            }
                        }
                    }
                    TransactionRequest::ID => {
                        debug!("[PeerWorker({})] Reading TransactionRequest...", self.peer.epid);
                        match TransactionRequest::from_full_bytes(&header, bytes) {
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
                                warn!(
                                    "[PeerWorker({})] Reading TransactionRequest failed: {:?}.",
                                    self.peer.epid, e
                                );

                                self.peer.metrics.invalid_messages_received_inc();
                                Protocol::get().metrics.invalid_messages_received_inc();
                            }
                        }
                    }
                    Heartbeat::ID => {
                        debug!("[PeerWorker({})] Reading Heartbeat...", self.peer.epid);
                        match Heartbeat::from_full_bytes(&header, bytes) {
                            Ok(message) => {
                                self.peer.set_solid_milestone_index(message.solid_milestone_index);
                                self.peer.set_snapshot_milestone_index(message.snapshot_milestone_index);

                                self.peer.metrics.heartbeat_received_inc();
                                Protocol::get().metrics.heartbeat_received_inc();
                            }
                            Err(e) => {
                                warn!("[PeerWorker({})] Reading Heartbeat failed: {:?}.", self.peer.epid, e);

                                self.peer.metrics.invalid_messages_received_inc();
                                Protocol::get().metrics.invalid_messages_received_inc();
                            }
                        }
                    }
                    _ => {
                        warn!("[PeerWorker({})] Ignoring unsupported message.", self.peer.epid);

                        self.peer.metrics.invalid_messages_received_inc();
                        Protocol::get().metrics.invalid_messages_received_inc();
                    }
                };
            }
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
                        debug!("[PeerWorker({})] Reading Header...", self.peer.epid);
                        let header = Header::from_bytes(&context.buffer[offset..offset + 3]);
                        offset = offset + 3;

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
                            error!("[PeerWorker({})] Processing message failed: {:?}.", self.peer.epid, e);
                        }

                        offset = offset + header.message_length as usize;

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
