use crate::message::{
    Handshake, Heartbeat, LegacyGossip, Message, MilestoneRequest, TransactionBroadcast, TransactionRequest,
};
use crate::neighbor::{NeighborEvent, NeighborSenderActor};
use crate::protocol::{COORDINATOR_BYTES, MINIMUM_WEIGHT_MAGNITUDE, SUPPORTED_VERSIONS};

use netzwerk::Command::SendBytes;
use netzwerk::{Network, PeerId};

use std::convert::TryInto;

use async_std::task::spawn;
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
    pub(crate) fn new(peer_id: PeerId, network: Network, receiver: Receiver<NeighborEvent>) -> Self {
        Self {
            peer_id: peer_id,
            network: network,
            receiver: receiver,
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
        let mut state = NeighborReceiverActorState::AwaitingConnection(AwaitingConnectionContext {});

        while let Some(event) = self.receiver.next().await {
            if let NeighborEvent::Removed = event {
                break;
            }

            state = match state {
                NeighborReceiverActorState::AwaitingConnection(context) => {
                    self.connection_handler(context, event).await
                }
                NeighborReceiverActorState::AwaitingHandshake(context) => self.handshake_handler(context, event).await,
                NeighborReceiverActorState::AwaitingMessage(context) => self.message_handler(context, event).await,
            }
        }
    }

    async fn connection_handler(
        &mut self,
        context: AwaitingConnectionContext,
        event: NeighborEvent,
    ) -> NeighborReceiverActorState {
        match event {
            NeighborEvent::Connected => {
                info!("[Neighbor-{:?}] Connected", self.peer_id);

                // TODO send handshake ?
                // TODO spawn ?
                self.send_handshake().await;

                NeighborReceiverActorState::AwaitingHandshake(AwaitingHandshakeContext { header: None })
            }
            _ => NeighborReceiverActorState::AwaitingConnection(context),
        }
    }

    fn check_handshake(&self, header: [u8; 3], bytes: &[u8]) -> NeighborReceiverActorState {
        info!("[Neighbor-{:?}] Reading Handshake", self.peer_id);

        let handshake = Some(Handshake::from_full_bytes(&header, bytes));

        // TODO check handshake

        NeighborReceiverActorState::AwaitingMessage(AwaitingMessageContext { header: None })
    }

    async fn handshake_handler(
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
                    NeighborReceiverActorState::AwaitingHandshake(AwaitingHandshakeContext { header: None })
                } else {
                    match context.header {
                        Some(header) => self.check_handshake(header, &bytes[3..size - 3]),
                        None => {
                            info!("[Neighbor-{:?}] Reading Header", self.peer_id);

                            let header: [u8; 3] = bytes[0..3].try_into().unwrap();

                            if size > 3 {
                                self.check_handshake(header, &bytes[3..size - 3])
                            } else {
                                NeighborReceiverActorState::AwaitingHandshake(AwaitingHandshakeContext {
                                    header: Some(header),
                                })
                            }
                        }
                    }
                }
            }
            _ => NeighborReceiverActorState::AwaitingHandshake(context),
        }
    }

    fn process_message() {
        // info!("[Neighbor ] Message received");
        // let header = Header::from_bytes(&bytes[0..size]).unwrap();
        // println!("{:?}", header.message_type());
        let message_type = 0x01;
        let message = [1];

        match message_type {
            Handshake::ID => {
                Handshake::from_bytes(&message);
            }
            LegacyGossip::ID => {
                LegacyGossip::from_bytes(&message);
            }
            MilestoneRequest::ID => {
                MilestoneRequest::from_bytes(&message);
            }
            TransactionBroadcast::ID => {
                TransactionBroadcast::from_bytes(&message);
            }
            TransactionRequest::ID => {
                TransactionRequest::from_bytes(&message);
            }
            Heartbeat::ID => {
                Heartbeat::from_bytes(&message);
            } // _ => Err(MessageError::InvalidMessageType(message_type)),
            _ => {}
        }
    }

    async fn message_handler(
        &mut self,
        context: AwaitingMessageContext,
        event: NeighborEvent,
    ) -> NeighborReceiverActorState {
        spawn(NeighborSenderActor::<LegacyGossip>::new(self.peer_id, self.network.clone()).run());
        spawn(NeighborSenderActor::<MilestoneRequest>::new(self.peer_id, self.network.clone()).run());
        spawn(NeighborSenderActor::<TransactionBroadcast>::new(self.peer_id, self.network.clone()).run());
        spawn(NeighborSenderActor::<TransactionRequest>::new(self.peer_id, self.network.clone()).run());
        spawn(NeighborSenderActor::<Heartbeat>::new(self.peer_id, self.network.clone()).run());

        match event {
            NeighborEvent::Disconnected => {
                info!("[Neighbor-{:?}] Disconnected", self.peer_id);

                NeighborReceiverActorState::AwaitingConnection(AwaitingConnectionContext {})
            }
            NeighborEvent::Message { size, bytes } => {
                info!("[Neighbor-{:?}] Message received", self.peer_id);

                if size < 3 {
                    NeighborReceiverActorState::AwaitingMessage(AwaitingMessageContext { header: None })
                } else {
                    match context.header {
                        Some(header) => {
                            NeighborReceiverActorState::AwaitingMessage(AwaitingMessageContext { header: None })
                        }
                        None => NeighborReceiverActorState::AwaitingMessage(AwaitingMessageContext { header: None }),
                    }
                }
            }
            _ => NeighborReceiverActorState::AwaitingMessage(context),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
}
