use crate::message::{Handshake, Header, Heartbeat, Message};
use crate::neighbor::NeighborSenders;
use crate::node::NodeMetrics;

use netzwerk::Command::SendBytes;
use netzwerk::{Network, PeerId};

use futures::channel::mpsc::Receiver;
use futures::stream::StreamExt;
use futures::{select, FutureExt};
use log::*;

pub(crate) struct Neighbor {
    pub(crate) senders: NeighborSenders,
    pub(crate) metrics: NodeMetrics,
    heartbeat: Heartbeat,
}

pub enum NeighborEvent {
    Connected,
    Disconnected,
    Message { size: usize, bytes: Vec<u8> },
}

impl Neighbor {
    pub fn new(senders: NeighborSenders) -> Self {
        Self {
            senders: senders,
            metrics: NodeMetrics::default(),
            heartbeat: Heartbeat::default(),
        }
    }

    // TODO pass sender as well
    pub async fn actor<M: Message>(mut receiver: Receiver<M>) {
        while let Some(message) = receiver.next().await {
            message.into_full_bytes();
            // TODO create event
            // TODO send to network
        }
    }

    pub async fn receive_actor(self) {}
}

pub(crate) trait NeighborReceiverActorState {}

pub(crate) struct GenericNeighborReceiverActor<S: NeighborReceiverActorState> {
    peer_id: PeerId,
    receiver: Receiver<NeighborEvent>,
    state: S,
}

pub(crate) struct NeighborConnectedReceiverActorState {
    // TODO state ?
    pub(crate) network: Network,
}
impl NeighborReceiverActorState for NeighborConnectedReceiverActorState {}

pub(crate) type NeighborReceiverActor =
    GenericNeighborReceiverActor<NeighborConnectedReceiverActorState>;

impl GenericNeighborReceiverActor<NeighborConnectedReceiverActorState> {
    pub(crate) fn new(
        peer_id: PeerId,
        receiver: Receiver<NeighborEvent>,
        state: NeighborConnectedReceiverActorState,
    ) -> Self {
        Self {
            peer_id: peer_id,
            receiver: receiver,
            state: state,
        }
    }

    pub(crate) async fn run(mut self) {
        while let Some(event) = self.receiver.next().await {
            match event {
                NeighborEvent::Connected => {
                    info!("[Neighbor ] Connected");
                    // TODO send handshake ?
                    println!("{:?}", (1337 as u16).to_be_bytes());
                    let bytes = [
                        1, 0, 61, 5, 57, 0, 0, 1, 112, 151, 168, 246, 60, 234, 56, 202, 174, 238,
                        197, 195, 253, 109, 14, 137, 227, 44, 144, 151, 188, 192, 45, 220, 236, 64,
                        168, 220, 197, 22, 199, 188, 1, 45, 11, 107, 190, 49, 84, 147, 176, 184,
                        108, 223, 189, 17, 167, 184, 240, 213, 170, 111, 34, 0, 14, 3,
                    ];
                    // TODO block ?
                    self.state
                        .network
                        .send(SendBytes {
                            to: self.peer_id,
                            bytes: bytes.to_vec(),
                        })
                        .await;
                    return GenericNeighborReceiverActor::<NeighborHandshakeReceiverActorState>::new(self.peer_id,
                        self.receiver,
                    )
                    .run()
                    .await;
                }
                _ => {}
            }
        }
    }
}

struct NeighborHandshakeReceiverActorState {}
impl NeighborReceiverActorState for NeighborHandshakeReceiverActorState {}

impl GenericNeighborReceiverActor<NeighborHandshakeReceiverActorState> {
    fn new(peer_id: PeerId, receiver: Receiver<NeighborEvent>) -> Self {
        Self {
            peer_id: peer_id,
            receiver: receiver,
            state: NeighborHandshakeReceiverActorState {},
        }
    }

    async fn run(mut self) {
        // TODO periodically send handshake ?
        let mut header = None;

        while let Some(event) = self.receiver.next().await {
            match event {
                NeighborEvent::Message { size, bytes } => {
                    info!("[Neighbor ] Message received");
                    match header {
                        Some(Header {
                            message_type,
                            message_length,
                        }) => {
                            info!("[Neighbor ] Reading Handshake");
                            let handshake = Handshake::from_bytes(&bytes[3..size - 3]);
                        }
                        None => {
                            info!("[Neighbor ] Reading Header");
                            header = Some(Header::from_bytes(&bytes[0..size]).unwrap());
                            if size > 3 {
                                info!("[Neighbor ] Reading Handshake");
                                let handshake = Handshake::from_bytes(&bytes[3..size - 3]);
                                for i in 0..64 {
                                    print!("{:?},", bytes[i]);
                                }
                                println!("");
                                return GenericNeighborReceiverActor::<
                                    NeighborMessageReceiverActorState,
                                >::new(
                                    self.peer_id, self.receiver
                                )
                                .run()
                                .await;
                            }
                        }
                    }
                }
                NeighborEvent::Disconnected => {}
                _ => {}
            }
        }
    }
}

struct NeighborMessageReceiverActorState {}
impl NeighborReceiverActorState for NeighborMessageReceiverActorState {}

impl GenericNeighborReceiverActor<NeighborMessageReceiverActorState> {
    fn new(peer_id: PeerId, receiver: Receiver<NeighborEvent>) -> Self {
        Self {
            peer_id: peer_id,
            receiver: receiver,
            state: NeighborMessageReceiverActorState {},
        }
    }

    async fn run(mut self) {
        while let Some(event) = self.receiver.next().await {
            match event {
                NeighborEvent::Message { size, bytes } => {
                    info!("[Neighbor ] Message received");
                    let header = Header::from_bytes(&bytes[0..size]).unwrap();
                    println!("{:?}", header.message_type());
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
}
