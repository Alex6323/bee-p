use crate::message::{Heartbeat, Message};
use crate::neighbor::NeighborSenders;
use crate::node::NodeMetrics;

use futures::channel::mpsc::Receiver;
use futures::stream::StreamExt;
use futures::{select, FutureExt};

pub(crate) struct Neighbor {
    pub(crate) senders: NeighborSenders,
    pub(crate) metrics: NodeMetrics,
    heartbeat: Heartbeat,
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
}

#[cfg(test)]
mod tests {

    use super::*;
}
