use crate::message::{Handshake, Message};
use crate::neighbor::NeighborEvent;

use netzwerk::Command::SendBytes;
use netzwerk::{Network, PeerId};

use std::convert::TryInto;

use futures::channel::mpsc::Receiver;
use futures::stream::StreamExt;
use log::*;

struct AwaitingConnectionContext {}
struct AwaitingHandshakeContext {
    header: Option<[u8; 3]>,
}
struct AwaitingMessageContext {
    header: Option<[u8; 3]>,
}

enum NeighborReceiverActorState {
    AwaitingConnection(AwaitingConnectionContext),
    AwaitingHandshake(AwaitingHandshakeContext),
    AwaitingMessage(AwaitingMessageContext),
}

pub(crate) struct NeighborReceiverActor {
    peer_id: PeerId,
    network: Network,
    receiver: Receiver<NeighborEvent>,
}

impl NeighborReceiverActor {
    pub(crate) fn new(
        peer_id: PeerId,
        network: Network,
        receiver: Receiver<NeighborEvent>,
    ) -> Self {
        Self {
            peer_id: peer_id,
            network: network,
            receiver: receiver,
        }
    }

    async fn awaiting_connection_handler(
        &mut self,
        context: AwaitingConnectionContext,
        event: NeighborEvent,
    ) -> NeighborReceiverActorState {
        match event {
            NeighborEvent::Connected => {
                info!("[Neighbor-{:?}] Connected", self.peer_id);

                // TODO send handshake ?
                let bytes = [
                    1, 0, 61, 5, 57, 0, 0, 1, 112, 151, 168, 246, 60, 234, 56, 202, 174, 238, 197,
                    195, 253, 109, 14, 137, 227, 44, 144, 151, 188, 192, 45, 220, 236, 64, 168,
                    220, 197, 22, 199, 188, 1, 45, 11, 107, 190, 49, 84, 147, 176, 184, 108, 223,
                    189, 17, 167, 184, 240, 213, 170, 111, 34, 0, 14, 3,
                ];
                // TODO block ?
                self.network
                    .send(SendBytes {
                        to_peer: self.peer_id,
                        bytes: bytes.to_vec(),
                    })
                    .await;

                NeighborReceiverActorState::AwaitingHandshake(AwaitingHandshakeContext {
                    header: None,
                })
            }
            _ => NeighborReceiverActorState::AwaitingConnection(context),
        }
    }

    async fn awaiting_handshake_handler(
        &mut self,
        context: AwaitingHandshakeContext,
        event: NeighborEvent,
    ) -> NeighborReceiverActorState {
        match event {
            NeighborEvent::Disconnected => {
                info!("[Neighbor-{:?}] Disconnected", self.peer_id);

                NeighborReceiverActorState::AwaitingConnection(AwaitingConnectionContext {})
            }
            NeighborEvent::Message { size, bytes } => {
                info!("[Neighbor-{:?}] Message received", self.peer_id);

                if size < 3 {
                    NeighborReceiverActorState::AwaitingHandshake(AwaitingHandshakeContext {
                        header: None,
                    })
                } else {
                    match context.header {
                        Some(header) => {
                            info!("[Neighbor-{:?}] Reading Handshake", self.peer_id);

                            let handshake =
                                Some(Handshake::from_full_bytes(&header, &bytes[3..size - 3]));

                            // TODO check handshake

                            NeighborReceiverActorState::AwaitingMessage(AwaitingMessageContext {
                                header: None,
                            })
                        }
                        None => {
                            info!("[Neighbor-{:?}] Reading Header", self.peer_id);

                            let header: [u8; 3] = bytes[0..3].try_into().unwrap();

                            if size > 3 {
                                info!("[Neighbor-{:?}] Reading Handshake", self.peer_id);

                                let handshake =
                                    Some(Handshake::from_full_bytes(&header, &bytes[3..size - 3]));

                                // TODO check handshake

                                NeighborReceiverActorState::AwaitingMessage(
                                    AwaitingMessageContext { header: None },
                                )
                            } else {
                                NeighborReceiverActorState::AwaitingHandshake(
                                    AwaitingHandshakeContext {
                                        header: Some(header),
                                    },
                                )
                            }
                        }
                    }
                }
            }
            _ => NeighborReceiverActorState::AwaitingHandshake(context),
        }
    }

    async fn awaiting_message_handler(
        &mut self,
        context: AwaitingMessageContext,
        event: NeighborEvent,
    ) -> NeighborReceiverActorState {
        match event {
            NeighborEvent::Disconnected => {
                info!("[Neighbor-{:?}] Disconnected", self.peer_id);

                NeighborReceiverActorState::AwaitingConnection(AwaitingConnectionContext {})
            }
            NeighborEvent::Message { size, bytes } => {
                info!("[Neighbor-{:?}] Message received", self.peer_id);

                if size < 3 {
                    NeighborReceiverActorState::AwaitingMessage(AwaitingMessageContext {
                        header: None,
                    })
                } else {
                    match context.header {
                        Some(header) => {
                            NeighborReceiverActorState::AwaitingMessage(AwaitingMessageContext {
                                header: None,
                            })
                        }
                        None => {
                            NeighborReceiverActorState::AwaitingMessage(AwaitingMessageContext {
                                header: None,
                            })
                        }
                    }
                }
            }
            _ => NeighborReceiverActorState::AwaitingMessage(context),
        }
    }

    pub(crate) async fn run(mut self) {
        let mut state =
            NeighborReceiverActorState::AwaitingConnection(AwaitingConnectionContext {});

        while let Some(event) = self.receiver.next().await {
            state = match state {
                NeighborReceiverActorState::AwaitingConnection(context) => {
                    self.awaiting_connection_handler(context, event).await
                }
                NeighborReceiverActorState::AwaitingHandshake(context) => {
                    self.awaiting_handshake_handler(context, event).await
                }
                NeighborReceiverActorState::AwaitingMessage(context) => {
                    self.awaiting_message_handler(context, event).await
                }
            }
        }
    }
}

// impl GenericNeighborReceiverActor<NeighborMessageReceiverActorState> {
//     async fn run(mut self) {
//         while let Some(event) = self.receiver.next().await {
//             match event {
//                 NeighborEvent::Message { size, bytes } => {
//                     // info!("[Neighbor ] Message received");
//                     // let header = Header::from_bytes(&bytes[0..size]).unwrap();
//                     // println!("{:?}", header.message_type());
//                     // match message_type {
//                     //     0x01 => Ok(ProtocolMessageType::Handshake(Handshake::from_bytes(
//                     //         &message,
//                     //     )?)),
//                     //     0x02 => Ok(ProtocolMessageType::LegacyGossip(LegacyGossip::from_bytes(
//                     //         &message,
//                     //     )?)),
//                     //     0x03 => Ok(ProtocolMessageType::MilestoneRequest(
//                     //         MilestoneRequest::from_bytes(&message)?,
//                     //     )),
//                     //     0x04 => Ok(ProtocolMessageType::TransactionBroadcast(
//                     //         TransactionBroadcast::from_bytes(&message)?,
//                     //     )),
//                     //     0x05 => Ok(ProtocolMessageType::TransactionRequest(
//                     //         TransactionRequest::from_bytes(&message)?,
//                     //     )),
//                     //     0x06 => Ok(ProtocolMessageType::Heartbeat(Heartbeat::from_bytes(
//                     //         &message,
//                     //     )?)),
//                     //     _ => Err(MessageError::InvalidMessageType(message_type)),
//                     // }
//                 }
//                 _ => {}
//             }
//         }
//     }
// }

#[cfg(test)]
mod tests {

    use super::*;
}
