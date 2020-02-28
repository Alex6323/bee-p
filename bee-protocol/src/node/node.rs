use crate::message::{
    Handshake, Heartbeat, LegacyGossip, MilestoneRequest, TransactionBroadcast, TransactionRequest,
};

// TODO channels ?
use crate::neighbor::{Neighbor, NeighborChannels};
use crate::node::NodeMetrics;

use std::collections::HashMap;

use async_std::task::{block_on, spawn};
use futures::channel::mpsc::SendError;
use futures::sink::SinkExt;

struct Node {
    // TODO thread-safety
    // TODO PeerID
    neighbors: HashMap<String, Neighbor>,
    metrics: NodeMetrics,
}

impl Node {
    fn new() -> Self {
        Self {
            neighbors: HashMap::new(),
            metrics: NodeMetrics::default(),
        }
    }

    async fn actor(self) {
        // while let Some(event) = events.next().await {
        //     match event {
        //         Event::BytesReceived {
        //             num_bytes,
        //             from,
        //             bytes,
        //         } => {}
        //         _ => (),
        //     }
        // }
    }

    fn start(self) {
        spawn(Self::actor(self));
    }

    fn added_neighbor(&mut self, id: String) {
        // let neighbor = Neighbor::new();
        // // TODO check return
        // self.neighbors.insert(id, neighbor);
        // spawn(Neighbor::actor(self.neighbors.get(&id).unwrap()));
    }

    pub async fn send_handshake(
        &self,
        neighbor: &mut Neighbor,
        handshake: Handshake,
    ) -> Result<(), SendError> {
        let res = neighbor.channels.handshake.0.send(handshake).await;

        if res.is_ok() {
            neighbor.metrics.handshake_sent_inc();
            self.metrics.handshake_sent_inc();
        }

        res
    }

    pub async fn send_legacy_gossip(
        &self,
        neighbor: &mut Neighbor,
        legacy_gossip: LegacyGossip,
    ) -> Result<(), SendError> {
        let res = neighbor.channels.legacy_gossip.0.send(legacy_gossip).await;

        if res.is_ok() {
            neighbor.metrics.transactions_sent_inc();
            neighbor.metrics.legacy_gossip_sent_inc();
            self.metrics.transactions_sent_inc();
            self.metrics.legacy_gossip_sent_inc();
        }

        res
    }

    pub async fn send_milestone_request(
        &self,
        neighbor: &mut Neighbor,
        milestone_request: MilestoneRequest,
    ) -> Result<(), SendError> {
        let res = neighbor
            .channels
            .milestone_request
            .0
            .send(milestone_request)
            .await;

        if res.is_ok() {
            neighbor.metrics.milestone_request_sent_inc();
            self.metrics.milestone_request_sent_inc();
        }

        res
    }

    pub async fn send_transaction_broadcast(
        &self,
        neighbor: &mut Neighbor,
        transaction_broadcast: TransactionBroadcast,
    ) -> Result<(), SendError> {
        let res = neighbor
            .channels
            .transaction_broadcast
            .0
            .send(transaction_broadcast)
            .await;

        if res.is_ok() {
            neighbor.metrics.transactions_sent_inc();
            neighbor.metrics.transaction_broadcast_sent_inc();
            self.metrics.transactions_sent_inc();
            self.metrics.transaction_broadcast_sent_inc();
        }

        res
    }

    pub async fn send_transaction_request(
        &self,
        neighbor: &mut Neighbor,
        transaction_request: TransactionRequest,
    ) -> Result<(), SendError> {
        let res = neighbor
            .channels
            .transaction_request
            .0
            .send(transaction_request)
            .await;

        if res.is_ok() {
            neighbor.metrics.transaction_request_sent_inc();
            self.metrics.transaction_request_sent_inc();
        }

        res
    }

    pub async fn send_heartbeat(
        &self,
        neighbor: &mut Neighbor,
        heartbeat: Heartbeat,
    ) -> Result<(), SendError> {
        let res = neighbor.channels.heartbeat.0.send(heartbeat).await;

        if res.is_ok() {
            neighbor.metrics.heartbeat_sent_inc();
            self.metrics.heartbeat_sent_inc();
        }

        res
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use futures::stream::{Stream, StreamExt};

    #[test]
    fn send_handshake_test() {
        let node = Node::new();
        let mut neighbor = Neighbor::new();

        assert_eq!(node.metrics.handshake_sent(), 0);
        assert_eq!(neighbor.metrics.handshake_sent(), 0);

        assert!(neighbor.channels.handshake.1.try_next().is_err());
        assert!(block_on(node.send_handshake(&mut neighbor, Handshake::default())).is_ok());
        assert!(block_on(neighbor.channels.handshake.1.next()).is_some());

        assert_eq!(node.metrics.handshake_sent(), 1);
        assert_eq!(neighbor.metrics.handshake_sent(), 1);
    }

    #[test]
    fn send_legacy_gossip_test() {
        let node = Node::new();
        let mut neighbor = Neighbor::new();

        assert_eq!(node.metrics.legacy_gossip_sent(), 0);
        assert_eq!(node.metrics.transactions_sent(), 0);
        assert_eq!(neighbor.metrics.legacy_gossip_sent(), 0);
        assert_eq!(neighbor.metrics.transactions_sent(), 0);

        assert!(neighbor.channels.legacy_gossip.1.try_next().is_err());
        assert!(block_on(node.send_legacy_gossip(&mut neighbor, LegacyGossip::default())).is_ok());
        assert!(block_on(neighbor.channels.legacy_gossip.1.next()).is_some());

        assert_eq!(node.metrics.legacy_gossip_sent(), 1);
        assert_eq!(node.metrics.transactions_sent(), 1);
        assert_eq!(neighbor.metrics.legacy_gossip_sent(), 1);
        assert_eq!(neighbor.metrics.transactions_sent(), 1);
    }

    #[test]
    fn send_milestone_request_test() {
        let node = Node::new();
        let mut neighbor = Neighbor::new();

        assert_eq!(node.metrics.milestone_request_sent(), 0);
        assert_eq!(neighbor.metrics.milestone_request_sent(), 0);

        assert!(neighbor.channels.milestone_request.1.try_next().is_err());
        assert!(
            block_on(node.send_milestone_request(&mut neighbor, MilestoneRequest::default()))
                .is_ok()
        );
        assert!(block_on(neighbor.channels.milestone_request.1.next()).is_some());

        assert_eq!(node.metrics.milestone_request_sent(), 1);
        assert_eq!(neighbor.metrics.milestone_request_sent(), 1);
    }

    #[test]
    fn send_transaction_broadcast_test() {
        let node = Node::new();
        let mut neighbor = Neighbor::new();

        assert_eq!(node.metrics.transaction_broadcast_sent(), 0);
        assert_eq!(node.metrics.transactions_sent(), 0);
        assert_eq!(neighbor.metrics.transaction_broadcast_sent(), 0);
        assert_eq!(neighbor.metrics.transactions_sent(), 0);

        assert!(neighbor
            .channels
            .transaction_broadcast
            .1
            .try_next()
            .is_err());
        assert!(block_on(
            node.send_transaction_broadcast(&mut neighbor, TransactionBroadcast::default())
        )
        .is_ok());
        assert!(block_on(neighbor.channels.transaction_broadcast.1.next()).is_some());

        assert_eq!(node.metrics.transaction_broadcast_sent(), 1);
        assert_eq!(node.metrics.transactions_sent(), 1);
        assert_eq!(neighbor.metrics.transaction_broadcast_sent(), 1);
        assert_eq!(neighbor.metrics.transactions_sent(), 1);
    }

    #[test]
    fn send_transaction_request_test() {
        let node = Node::new();
        let mut neighbor = Neighbor::new();

        assert_eq!(node.metrics.transaction_request_sent(), 0);
        assert_eq!(neighbor.metrics.transaction_request_sent(), 0);

        assert!(neighbor.channels.transaction_request.1.try_next().is_err());
        assert!(block_on(
            node.send_transaction_request(&mut neighbor, TransactionRequest::default())
        )
        .is_ok());
        assert!(block_on(neighbor.channels.transaction_request.1.next()).is_some());

        assert_eq!(node.metrics.transaction_request_sent(), 1);
        assert_eq!(neighbor.metrics.transaction_request_sent(), 1);
    }

    #[test]
    fn send_heartbeat_test() {
        let node = Node::new();
        let mut neighbor = Neighbor::new();

        assert_eq!(node.metrics.heartbeat_sent(), 0);
        assert_eq!(neighbor.metrics.heartbeat_sent(), 0);

        assert!(neighbor.channels.heartbeat.1.try_next().is_err());
        assert!(block_on(node.send_heartbeat(&mut neighbor, Heartbeat::default())).is_ok());
        assert!(block_on(neighbor.channels.heartbeat.1.next()).is_some());

        assert_eq!(node.metrics.heartbeat_sent(), 1);
        assert_eq!(neighbor.metrics.heartbeat_sent(), 1);
    }
}
