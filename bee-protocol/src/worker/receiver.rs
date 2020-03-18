use crate::message::{
    Handshake, Header, Heartbeat, LegacyGossip, Message, MilestoneRequest, TransactionBroadcast, TransactionRequest,
};
use crate::protocol::{COORDINATOR_BYTES, MINIMUM_WEIGHT_MAGNITUDE, SUPPORTED_VERSIONS};
use crate::worker::ResponderWorkerEvent;

use bee_network::Command::SendBytes;
use bee_network::{EndpointId, Network};

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
    Message(Vec<u8>),
}

enum ReceiverWorkerMessageState {
    Header { offset: usize },
    Payload { header: Header, offset: usize },
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
    epid: EndpointId,
    network: Network,
    receiver: Receiver<ReceiverWorkerEvent>,
    transaction_worker_sender: Sender<TransactionBroadcast>,
    responder_worker: Sender<ResponderWorkerEvent>,
}

impl ReceiverWorker {
    pub(crate) fn new(
        epid: EndpointId,
        network: Network,
        receiver: Receiver<ReceiverWorkerEvent>,
        transaction_worker_sender: Sender<TransactionBroadcast>,
        responder_worker: Sender<ResponderWorkerEvent>,
    ) -> Self {
        Self {
            epid: epid,
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
                epid: self.epid,
                bytes: bytes.to_vec(),
                responder: None,
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
                info!("[Neighbor-{:?}] Connected", self.epid);

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
        debug!("[Neighbor-{:?}] Reading Handshake", self.epid);

        match Handshake::from_full_bytes(&header, bytes) {
            Ok(handshake) => {
                // TODO check handshake

                // if handshake.port != port {
                //     warn!(
                //         "[Neighbor-{:?}] Invalid handshake port: {:?} != {:?}",
                //         self.epid, handshake.port, port
                //     );
                // } else if handshake.timestamp != timestamp {
                //     warn!(
                //         "[Neighbor-{:?}] Invalid handshake timestamp: {:?}",
                //         self.epid, handshake.timestamp
                //     );
                // } else if handshake.coordinator != coordinator {
                //     warn!("[Neighbor-{:?}] Invalid handshake coordinator", self.epid);
                // } else if handshake.minimum_weight_magnitude != minimum_weight_magnitude {
                //     warn!(
                //         "[Neighbor-{:?}] Invalid handshake MWM: {:?} != {:?}",
                //         self.epid, handshake.minimum_weight_magnitude, minimum_weight_magnitude
                //     );
                // } else if let Err(version) = supported_version(handshake.supported_messages) {
                //     warn!(
                //         "[Neighbor-{:?}] Unsupported protocol version: {:?}",
                //         self.epid, version
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

                info!("[Neighbor-{:?}] Handshake completed", self.epid);

                ReceiverWorkerState::AwaitingMessage(AwaitingMessageContext {
                    state: ReceiverWorkerMessageState::Header { offset: 0 },
                })
            }

            Err(e) => {
                warn!("[Neighbor-{:?}] Reading Handshake failed: {:?}", self.epid, e);

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
                info!("[Neighbor-{:?}] Disconnected", self.epid);

                ReceiverWorkerState::AwaitingConnection(AwaitingConnectionContext {})
            }
            ReceiverWorkerEvent::Message(bytes) => {
                // TODO needed ?
                if bytes.len() < 3 {
                    ReceiverWorkerState::AwaitingHandshake(AwaitingHandshakeContext {
                        state: ReceiverWorkerMessageState::Header { offset: 0 },
                    })
                } else {
                    match context.state {
                        ReceiverWorkerMessageState::Header { .. } => {
                            debug!("[Neighbor-{:?}] Reading Header", self.epid);

                            let header = Header::from_bytes(&bytes[0..3]);

                            if bytes.len() > 3 {
                                self.check_handshake(header, &bytes[3..bytes.len()])
                            } else {
                                ReceiverWorkerState::AwaitingHandshake(AwaitingHandshakeContext {
                                    state: ReceiverWorkerMessageState::Payload {
                                        header: header,
                                        offset: 0,
                                    },
                                })
                            }
                        }
                        ReceiverWorkerMessageState::Payload { header, offset } => {
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
                warn!("[Neighbor-{:?}] Ignoring unexpected Handshake", self.epid);
                // TODO handle here instead of dedicated state ?
            }

            LegacyGossip::ID => {
                warn!("[Neighbor-{:?}] Ignoring unsupported LegacyGossip", self.epid);
            }

            MilestoneRequest::ID => {
                debug!("[Neighbor-{:?}] Receiving MilestoneRequest", self.epid);

                match MilestoneRequest::from_full_bytes(&header, bytes) {
                    Ok(message) => {
                        self.responder_worker
                            .send(ResponderWorkerEvent::MilestoneRequest {
                                epid: self.epid,
                                message: message,
                            })
                            .await
                            .map_err(|_| ReceiverWorkerError::FailedSend)?;
                    }
                    Err(e) => {
                        warn!("[Neighbor-{:?}] Receiving MilestoneRequest failed: {:?}", self.epid, e);
                    }
                }
            }

            TransactionBroadcast::ID => {
                debug!("[Neighbor-{:?}] Receiving TransactionBroadcast", self.epid);

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
                            self.epid, e
                        );
                    }
                }
            }

            TransactionRequest::ID => {
                debug!("[Neighbor-{:?}] Receiving TransactionRequest", self.epid);

                match TransactionRequest::from_full_bytes(&header, bytes) {
                    Ok(message) => {
                        self.responder_worker
                            .send(ResponderWorkerEvent::TransactionRequest {
                                epid: self.epid,
                                message: message,
                            })
                            .await
                            .map_err(|_| ReceiverWorkerError::FailedSend)?;
                    }
                    Err(e) => {
                        warn!(
                            "[Neighbor-{:?}] Receiving TransactionRequest failed: {:?}",
                            self.epid, e
                        );
                    }
                }
            }

            Heartbeat::ID => {
                debug!("[Neighbor-{:?}] Receiving Heartbeat", self.epid);

                match Heartbeat::from_full_bytes(&header, bytes) {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("[Neighbor-{:?}] Receiving Heartbeat failed: {:?}", self.epid, e);
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
        // spawn(SenderWorker::<LegacyGossip>::new(self.epid, self.network.clone()).run());
        // spawn(SenderWorker::<MilestoneRequest>::new(self.epid, self.network.clone()).run());
        // spawn(SenderWorker::<TransactionBroadcast>::new(self.epid, self.network.clone()).run());
        // spawn(SenderWorker::<TransactionRequest>::new(self.epid, self.network.clone()).run());
        // spawn(SenderWorker::<Heartbeat>::new(self.epid, self.network.clone()).run());

        match event {
            ReceiverWorkerEvent::Disconnected => {
                debug!("[Neighbor-{:?}] Disconnected", self.epid);

                ReceiverWorkerState::AwaitingConnection(AwaitingConnectionContext {})
            }
            ReceiverWorkerEvent::Message(bytes) => {
                // TODO needed ?
                if bytes.len() < 3 {
                    ReceiverWorkerState::AwaitingMessage(AwaitingMessageContext {
                        state: ReceiverWorkerMessageState::Header { offset: 0 },
                    })
                } else {
                    loop {
                        context.state = match context.state {
                            ReceiverWorkerMessageState::Header { offset } => {
                                debug!("[Neighbor-{:?}] Reading Header", self.epid);

                                if offset as usize == bytes.len() {
                                    break ReceiverWorkerState::AwaitingMessage(AwaitingMessageContext {
                                        state: ReceiverWorkerMessageState::Header { offset: 0 },
                                    });
                                }

                                ReceiverWorkerMessageState::Payload {
                                    header: Header::from_bytes(&bytes[offset..offset + 3]),
                                    offset: offset + 3,
                                }
                            }
                            ReceiverWorkerMessageState::Payload { header, offset } => {
                                // TODO check that size is enough

                                if offset as usize == bytes.len() {
                                    break ReceiverWorkerState::AwaitingMessage(AwaitingMessageContext {
                                        state: ReceiverWorkerMessageState::Payload {
                                            header: header,
                                            offset: 0,
                                        },
                                    });
                                }

                                if let Err(e) = self
                                    .process_message(&header, &bytes[offset..offset + header.message_length as usize])
                                    .await
                                {
                                    error!("[Neighbor-{:?}] Processing message failed: {:?}", self.epid, e);
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
