use crate::message::Heartbeat;
use crate::neighbor::NeighborQueues;
use crate::node::NodeMetrics;

use bee_common::logger;

use futures::stream::StreamExt;
use futures::{select, FutureExt};

#[derive(Default)]
pub(crate) struct Neighbor {
    pub(crate) queues: NeighborQueues,
    pub(crate) metrics: NodeMetrics,
    heartbeat: Heartbeat,
}

impl Neighbor {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn actor(&mut self) {
        logger::debug("Neighbor actor launched for [PeerID]");

        loop {
            select! {
                message = self.queues.handshake.1.next().fuse() => (),
                message = self.queues.legacy_gossip.1.next().fuse() => (),
                message = self.queues.milestone_request.1.next().fuse() => (),
                message = self.queues.transaction_broadcast.1.next().fuse() => (),
                message = self.queues.transaction_request.1.next().fuse() => (),
                message = self.queues.heartbeat.1.next().fuse() => (),
            };
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn neighbor_test() {
        let neighbor = Neighbor::new();
    }
}
