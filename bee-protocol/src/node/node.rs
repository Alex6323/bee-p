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

pub struct Node {
    // TODO thread-safety
    // TODO PeerID
    neighbors: HashMap<String, Neighbor>,
    metrics: NodeMetrics,
}

impl Node {
    pub fn new() -> Self {
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
        let channels = NeighborChannels::new();
        let neighbor = Neighbor::new(channels.senders);

        // // TODO check return
        // TODO remove clone
        self.neighbors.insert(id.clone(), neighbor);

        spawn(Neighbor::actor::<Handshake>(channels.receivers.handshake));
        spawn(Neighbor::actor::<LegacyGossip>(
            channels.receivers.legacy_gossip,
        ));
        spawn(Neighbor::actor::<MilestoneRequest>(
            channels.receivers.milestone_request,
        ));
        spawn(Neighbor::actor::<TransactionBroadcast>(
            channels.receivers.transaction_broadcast,
        ));
        spawn(Neighbor::actor::<TransactionRequest>(
            channels.receivers.transaction_request,
        ));
        spawn(Neighbor::actor::<Heartbeat>(channels.receivers.heartbeat));
    }

    async fn send_handshake(
        &self,
        neighbor: &mut Neighbor,
        handshake: Handshake,
    ) -> Result<(), SendError> {
        let res = neighbor.senders.handshake.send(handshake).await;

        if res.is_ok() {
            neighbor.metrics.handshake_sent_inc();
            self.metrics.handshake_sent_inc();
        }

        res
    }

    async fn send_legacy_gossip(
        &self,
        neighbor: &mut Neighbor,
        legacy_gossip: LegacyGossip,
    ) -> Result<(), SendError> {
        let res = neighbor.senders.legacy_gossip.send(legacy_gossip).await;

        if res.is_ok() {
            neighbor.metrics.transactions_sent_inc();
            neighbor.metrics.legacy_gossip_sent_inc();
            self.metrics.transactions_sent_inc();
            self.metrics.legacy_gossip_sent_inc();
        }

        res
    }

    async fn send_milestone_request(
        &self,
        neighbor: &mut Neighbor,
        milestone_request: MilestoneRequest,
    ) -> Result<(), SendError> {
        let res = neighbor
            .senders
            .milestone_request
            .send(milestone_request)
            .await;

        if res.is_ok() {
            neighbor.metrics.milestone_request_sent_inc();
            self.metrics.milestone_request_sent_inc();
        }

        res
    }

    async fn send_transaction_broadcast(
        &self,
        neighbor: &mut Neighbor,
        transaction_broadcast: TransactionBroadcast,
    ) -> Result<(), SendError> {
        let res = neighbor
            .senders
            .transaction_broadcast
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

    async fn send_transaction_request(
        &self,
        neighbor: &mut Neighbor,
        transaction_request: TransactionRequest,
    ) -> Result<(), SendError> {
        let res = neighbor
            .senders
            .transaction_request
            .send(transaction_request)
            .await;

        if res.is_ok() {
            neighbor.metrics.transaction_request_sent_inc();
            self.metrics.transaction_request_sent_inc();
        }

        res
    }

    async fn send_heartbeat(
        &self,
        neighbor: &mut Neighbor,
        heartbeat: Heartbeat,
    ) -> Result<(), SendError> {
        let res = neighbor.senders.heartbeat.send(heartbeat).await;

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
        let mut channels = NeighborChannels::new();
        let mut neighbor = Neighbor::new(channels.senders);

        assert_eq!(node.metrics.handshake_sent(), 0);
        assert_eq!(neighbor.metrics.handshake_sent(), 0);

        assert!(channels.receivers.handshake.try_next().is_err());
        assert!(block_on(node.send_handshake(&mut neighbor, Handshake::default())).is_ok());
        assert!(block_on(channels.receivers.handshake.next()).is_some());

        assert_eq!(node.metrics.handshake_sent(), 1);
        assert_eq!(neighbor.metrics.handshake_sent(), 1);
    }

    #[test]
    fn send_legacy_gossip_test() {
        let node = Node::new();
        let mut channels = NeighborChannels::new();
        let mut neighbor = Neighbor::new(channels.senders);

        assert_eq!(node.metrics.legacy_gossip_sent(), 0);
        assert_eq!(node.metrics.transactions_sent(), 0);
        assert_eq!(neighbor.metrics.legacy_gossip_sent(), 0);
        assert_eq!(neighbor.metrics.transactions_sent(), 0);

        assert!(channels.receivers.legacy_gossip.try_next().is_err());
        assert!(block_on(node.send_legacy_gossip(&mut neighbor, LegacyGossip::default())).is_ok());
        assert!(block_on(channels.receivers.legacy_gossip.next()).is_some());

        assert_eq!(node.metrics.legacy_gossip_sent(), 1);
        assert_eq!(node.metrics.transactions_sent(), 1);
        assert_eq!(neighbor.metrics.legacy_gossip_sent(), 1);
        assert_eq!(neighbor.metrics.transactions_sent(), 1);
    }

    #[test]
    fn send_milestone_request_test() {
        let node = Node::new();
        let mut channels = NeighborChannels::new();
        let mut neighbor = Neighbor::new(channels.senders);

        assert_eq!(node.metrics.milestone_request_sent(), 0);
        assert_eq!(neighbor.metrics.milestone_request_sent(), 0);

        assert!(channels.receivers.milestone_request.try_next().is_err());
        assert!(
            block_on(node.send_milestone_request(&mut neighbor, MilestoneRequest::default()))
                .is_ok()
        );
        assert!(block_on(channels.receivers.milestone_request.next()).is_some());

        assert_eq!(node.metrics.milestone_request_sent(), 1);
        assert_eq!(neighbor.metrics.milestone_request_sent(), 1);
    }

    #[test]
    fn send_transaction_broadcast_test() {
        let node = Node::new();
        let mut channels = NeighborChannels::new();
        let mut neighbor = Neighbor::new(channels.senders);

        assert_eq!(node.metrics.transaction_broadcast_sent(), 0);
        assert_eq!(node.metrics.transactions_sent(), 0);
        assert_eq!(neighbor.metrics.transaction_broadcast_sent(), 0);
        assert_eq!(neighbor.metrics.transactions_sent(), 0);

        assert!(channels.receivers.transaction_broadcast.try_next().is_err());
        assert!(block_on(
            node.send_transaction_broadcast(&mut neighbor, TransactionBroadcast::default())
        )
        .is_ok());
        assert!(block_on(channels.receivers.transaction_broadcast.next()).is_some());

        assert_eq!(node.metrics.transaction_broadcast_sent(), 1);
        assert_eq!(node.metrics.transactions_sent(), 1);
        assert_eq!(neighbor.metrics.transaction_broadcast_sent(), 1);
        assert_eq!(neighbor.metrics.transactions_sent(), 1);
    }

    #[test]
    fn send_transaction_request_test() {
        let node = Node::new();
        let mut channels = NeighborChannels::new();
        let mut neighbor = Neighbor::new(channels.senders);

        assert_eq!(node.metrics.transaction_request_sent(), 0);
        assert_eq!(neighbor.metrics.transaction_request_sent(), 0);

        assert!(channels.receivers.transaction_request.try_next().is_err());
        assert!(block_on(
            node.send_transaction_request(&mut neighbor, TransactionRequest::default())
        )
        .is_ok());
        assert!(block_on(channels.receivers.transaction_request.next()).is_some());

        assert_eq!(node.metrics.transaction_request_sent(), 1);
        assert_eq!(neighbor.metrics.transaction_request_sent(), 1);
    }

    #[test]
    fn send_heartbeat_test() {
        let node = Node::new();
        let mut channels = NeighborChannels::new();
        let mut neighbor = Neighbor::new(channels.senders);

        assert_eq!(node.metrics.heartbeat_sent(), 0);
        assert_eq!(neighbor.metrics.heartbeat_sent(), 0);

        assert!(channels.receivers.heartbeat.try_next().is_err());
        assert!(block_on(node.send_heartbeat(&mut neighbor, Heartbeat::default())).is_ok());
        assert!(block_on(channels.receivers.heartbeat.next()).is_some());

        assert_eq!(node.metrics.heartbeat_sent(), 1);
        assert_eq!(neighbor.metrics.heartbeat_sent(), 1);
    }
}
