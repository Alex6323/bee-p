use crate::message::{
    Handshake, Header, Heartbeat, LegacyGossip, Message, MilestoneRequest, TransactionBroadcast, TransactionRequest,
};
use crate::protocol::{COORDINATOR_BYTES, MINIMUM_WEIGHT_MAGNITUDE, SUPPORTED_VERSIONS};
use crate::worker::ResponderWorkerEvent;

use netzwerk::Command::SendBytes;
use netzwerk::{Network, PeerId};

use futures::channel::mpsc::{Receiver, Sender};
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use log::*;

#[derive(Debug)]
pub(crate) enum ReceiverWorkerError {
    FailedSend,
}

pub(crate) enum ReceiverWorkerEvent {
    Removed,
    Connected,
    Disconnected,
    Message { size: usize, bytes: Vec<u8> },
}

enum ReceiverWorkerMessageState {
    Header { offset: usize },
    Payload { offset: usize, header: Header },
}

struct AwaitingConnectionContext {}

struct AwaitingHandshakeContext {
    state: ReceiverWorkerMessageState,
}

struct AwaitingMessageContext {
    state: ReceiverWorkerMessageState,
}

enum ReceiverWorkerState {
    AwaitingConnection(AwaitingConnectionContext),
    AwaitingHandshake(AwaitingHandshakeContext),
    AwaitingMessage(AwaitingMessageContext),
}

pub(crate) struct ReceiverWorker {
    peer_id: PeerId,
    network: Network,
    receiver: Receiver<ReceiverWorkerEvent>,
    transaction_worker_sender: Sender<TransactionBroadcast>,
    responder_worker: Sender<ResponderWorkerEvent>,
}

impl ReceiverWorker {
    pub(crate) fn new(
        peer_id: PeerId,
        network: Network,
        receiver: Receiver<ReceiverWorkerEvent>,
        transaction_worker_sender: Sender<TransactionBroadcast>,
        responder_worker: Sender<ResponderWorkerEvent>,
    ) -> Self {
        Self {
            peer_id: peer_id,
            network: network,
            receiver: receiver,
            transaction_worker_sender: transaction_worker_sender,
            responder_worker: responder_worker,
        }
    }

    async fn send_handshake(&mut self) {
        // TODO metric ?
        // TODO port
        let bytes =
            Handshake::new(1337, &COORDINATOR_BYTES, MINIMUM_WEIGHT_MAGNITUDE, &SUPPORTED_VERSIONS).into_full_bytes();

        self.network
            .send(SendBytes {
                to_peer: self.peer_id,
                bytes: bytes.to_vec(),
            })
            .await;
    }

    pub(crate) async fn run(mut self) {
        let mut state = ReceiverWorkerState::AwaitingConnection(AwaitingConnectionContext {});

        while let Some(event) = self.receiver.next().await {
            if let ReceiverWorkerEvent::Removed = event {
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
                info!("[Neighbor-{:?}] Connected", self.peer_id);

                // TODO spawn ?
                self.send_handshake().await;

                ReceiverWorkerState::AwaitingHandshake(AwaitingHandshakeContext {
                    state: ReceiverWorkerMessageState::Header { offset: 0 },
                })
            }
            _ => ReceiverWorkerState::AwaitingConnection(context),
        }
    }

    fn check_handshake(&self, header: Header, bytes: &[u8]) -> ReceiverWorkerState {
        info!("[Neighbor-{:?}] Reading Handshake", self.peer_id);

        match Handshake::from_full_bytes(&header, bytes) {
            Ok(handshake) => {
                // TODO check handshake

                // if handshake.port != port {
                //     warn!(
                //         "[Neighbor-{:?}] Invalid handshake port: {:?} != {:?}",
                //         self.peer_id, handshake.port, port
                //     );
                // } else if handshake.timestamp != timestamp {
                //     warn!(
                //         "[Neighbor-{:?}] Invalid handshake timestamp: {:?}",
                //         self.peer_id, handshake.timestamp
                //     );
                // } else if handshake.coordinator != coordinator {
                //     warn!("[Neighbor-{:?}] Invalid handshake coordinator", self.peer_id);
                // } else if handshake.minimum_weight_magnitude != minimum_weight_magnitude {
                //     warn!(
                //         "[Neighbor-{:?}] Invalid handshake MWM: {:?} != {:?}",
                //         self.peer_id, handshake.minimum_weight_magnitude, minimum_weight_magnitude
                //     );
                // } else if handshake.supported_messages != supported_messages {
                //     warn!(
                //         "[Neighbor-{:?}] Invalid handshake version: {:?}",
                //         self.peer_id, handshake.
                //     );
                // } else {
                //     ReceiverWorkerState::AwaitingMessage(AwaitingMessageContext {
                //         state: ReceiverWorkerMessageState::Header { offset: 0 },
                //     })
                // }
                //
                // ReceiverWorkerState::AwaitingHandshake(AwaitingHandshakeContext {
                //     state: ReceiverWorkerMessageState::Header { offset: 0 },
                // })

                ReceiverWorkerState::AwaitingMessage(AwaitingMessageContext {
                    state: ReceiverWorkerMessageState::Header { offset: 0 },
                })
            }

            Err(e) => {
                warn!("[Neighbor-{:?}] Reading Handshake failed: {:?}", self.peer_id, e);

                ReceiverWorkerState::AwaitingHandshake(AwaitingHandshakeContext {
                    state: ReceiverWorkerMessageState::Header { offset: 0 },
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
                info!("[Neighbor-{:?}] Disconnected", self.peer_id);

                ReceiverWorkerState::AwaitingConnection(AwaitingConnectionContext {})
            }
            ReceiverWorkerEvent::Message { size, bytes } => {
                // TODO needed ?
                if size < 3 {
                    ReceiverWorkerState::AwaitingHandshake(AwaitingHandshakeContext {
                        state: ReceiverWorkerMessageState::Header { offset: 0 },
                    })
                } else {
                    match context.state {
                        ReceiverWorkerMessageState::Header { .. } => {
                            info!("[Neighbor-{:?}] Reading Header", self.peer_id);

                            let header = Header::from_bytes(&bytes[0..3]);

                            if size > 3 {
                                self.check_handshake(header, &bytes[3..size])
                            } else {
                                ReceiverWorkerState::AwaitingHandshake(AwaitingHandshakeContext {
                                    state: ReceiverWorkerMessageState::Payload {
                                        offset: 0,
                                        header: header,
                                    },
                                })
                            }
                        }
                        ReceiverWorkerMessageState::Payload { offset, header } => {
                            self.check_handshake(header, &bytes[..size])
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
                warn!("[Neighbor-{:?}] Ignoring unexpected Handshake", self.peer_id);
                // TODO handle here instead of dedicated state ?
            }

            LegacyGossip::ID => {
                warn!("[Neighbor-{:?}] Ignoring unsupported LegacyGossip", self.peer_id);
            }

            MilestoneRequest::ID => {
                info!("[Neighbor-{:?}] Receiving MilestoneRequest", self.peer_id);

                match MilestoneRequest::from_full_bytes(&header, bytes) {
                    Ok(message) => {
                        self.responder_worker
                            .send(ResponderWorkerEvent::MilestoneRequest {
                                peer_id: self.peer_id,
                                message: message,
                            })
                            .await
                            .map_err(|_| ReceiverWorkerError::FailedSend)?;
                    }
                    Err(e) => {
                        warn!(
                            "[Neighbor-{:?}] Receiving MilestoneRequest failed: {:?}",
                            self.peer_id, e
                        );
                    }
                }
            }

            TransactionBroadcast::ID => {
                info!("[Neighbor-{:?}] Receiving TransactionBroadcast", self.peer_id);

                match TransactionBroadcast::from_full_bytes(&header, bytes) {
                    Ok(message) => {
                        self.transaction_worker_sender
                            .send(message)
                            .await
                            .map_err(|_| ReceiverWorkerError::FailedSend)?;
                    }
                    Err(e) => {
                        warn!(
                            "[Neighbor-{:?}] Receiving TransactionBroadcast failed: {:?}",
                            self.peer_id, e
                        );
                    }
                }
            }

            TransactionRequest::ID => {
                info!("[Neighbor-{:?}] Receiving TransactionRequest", self.peer_id);

                match TransactionRequest::from_full_bytes(&header, bytes) {
                    Ok(message) => {
                        self.responder_worker
                            .send(ResponderWorkerEvent::TransactionRequest {
                                peer_id: self.peer_id,
                                message: message,
                            })
                            .await
                            .map_err(|_| ReceiverWorkerError::FailedSend)?;
                    }
                    Err(e) => {
                        warn!(
                            "[Neighbor-{:?}] Receiving TransactionRequest failed: {:?}",
                            self.peer_id, e
                        );
                    }
                }
            }

            Heartbeat::ID => {
                info!("[Neighbor-{:?}] Receiving Heartbeat", self.peer_id);

                match Heartbeat::from_full_bytes(&header, bytes) {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("[Neighbor-{:?}] Receiving Heartbeat failed: {:?}", self.peer_id, e);
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
        // spawn(SenderWorker::<LegacyGossip>::new(self.peer_id, self.network.clone()).run());
        // spawn(SenderWorker::<MilestoneRequest>::new(self.peer_id, self.network.clone()).run());
        // spawn(SenderWorker::<TransactionBroadcast>::new(self.peer_id, self.network.clone()).run());
        // spawn(SenderWorker::<TransactionRequest>::new(self.peer_id, self.network.clone()).run());
        // spawn(SenderWorker::<Heartbeat>::new(self.peer_id, self.network.clone()).run());

        match event {
            ReceiverWorkerEvent::Disconnected => {
                info!("[Neighbor-{:?}] Disconnected", self.peer_id);

                ReceiverWorkerState::AwaitingConnection(AwaitingConnectionContext {})
            }
            ReceiverWorkerEvent::Message { size, bytes } => {
                // TODO needed ?
                if size < 3 {
                    ReceiverWorkerState::AwaitingMessage(AwaitingMessageContext {
                        state: ReceiverWorkerMessageState::Header { offset: 0 },
                    })
                } else {
                    loop {
                        context.state = match context.state {
                            ReceiverWorkerMessageState::Header { offset } => {
                                info!("[Neighbor-{:?}] Reading Header", self.peer_id);

                                if offset as usize == size {
                                    break ReceiverWorkerState::AwaitingMessage(AwaitingMessageContext {
                                        state: ReceiverWorkerMessageState::Header { offset: 0 },
                                    });
                                }

                                ReceiverWorkerMessageState::Payload {
                                    offset: offset + 3,
                                    header: Header::from_bytes(&bytes[offset..offset + 3]),
                                }
                            }
                            ReceiverWorkerMessageState::Payload { offset, header } => {
                                // TODO check that size is enough

                                if offset as usize == size {
                                    break ReceiverWorkerState::AwaitingMessage(AwaitingMessageContext {
                                        state: ReceiverWorkerMessageState::Payload {
                                            offset: 0,
                                            header: header,
                                        },
                                    });
                                }

                                if let Err(e) = self
                                    .process_message(&header, &bytes[offset..offset + header.message_length as usize])
                                    .await
                                {
                                    error!("[Neighbor-{:?}] Processing message failed: {:?}", self.peer_id, e);
                                }

                                ReceiverWorkerMessageState::Header {
                                    offset: offset + header.message_length as usize,
                                }
                            }
                        };
                    }
                }
            }
            _ => ReceiverWorkerState::AwaitingMessage(context),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
}
