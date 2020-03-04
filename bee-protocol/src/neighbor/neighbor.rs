use crate::message::{Handshake, Heartbeat, Message};
use crate::neighbor::NeighborSenders;
use crate::node::NodeMetrics;

use futures::{select, FutureExt};

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

    // // TODO pass sender as well
    // pub async fn actor<M: Message>(mut receiver: Receiver<M>) {
    //     while let Some(message) = receiver.next().await {
    //         message.into_full_bytes();
    //         // TODO create event
    //         // TODO send to network
    //     }
    // }
    //
    // pub async fn receive_actor(self) {}
}
