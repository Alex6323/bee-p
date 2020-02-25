use crate::message::{
    Handshake, Heartbeat, LegacyGossip, MilestoneRequest, TransactionBroadcast, TransactionRequest,
};
use crate::neighbor::NeighborQueues;
use crate::node::NodeMetrics;

use futures::channel::mpsc::SendError;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use futures::{select, FutureExt};

#[derive(Default)]
pub(crate) struct Neighbor {
    queues: NeighborQueues,
    metrics: NodeMetrics,
    heartbeat: Heartbeat,
}

impl Neighbor {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn send_handshake(&mut self, handshake: Handshake) -> Result<(), SendError> {
        // TODO add server metrics
        self.metrics.handshake_sent_inc();

        self.queues.handshake.0.send(handshake).await
    }

    pub async fn send_legacy_gossip(
        &mut self,
        legacy_gossip: LegacyGossip,
    ) -> Result<(), SendError> {
        // TODO add server metrics
        self.metrics.legacy_gossip_sent_inc();

        self.queues.legacy_gossip.0.send(legacy_gossip).await
    }

    pub async fn send_milestone_request(
        &mut self,
        milestone_request: MilestoneRequest,
    ) -> Result<(), SendError> {
        // TODO add server metrics
        self.metrics.milestone_request_sent_inc();

        self.queues
            .milestone_request
            .0
            .send(milestone_request)
            .await
    }

    pub async fn send_transaction_broadcast(
        &mut self,
        transaction_broadcast: TransactionBroadcast,
    ) -> Result<(), SendError> {
        // TODO add server metrics
        self.metrics.transaction_broadcast_sent_inc();

        self.queues
            .transaction_broadcast
            .0
            .send(transaction_broadcast)
            .await
    }

    pub async fn send_transaction_request(
        &mut self,
        transaction_request: TransactionRequest,
    ) -> Result<(), SendError> {
        // TODO add server metrics
        self.metrics.transaction_request_sent_inc();

        self.queues
            .transaction_request
            .0
            .send(transaction_request)
            .await
    }

    pub async fn send_heartbeat(&mut self, heartbeat: Heartbeat) -> Result<(), SendError> {
        // TODO add server metrics
        self.metrics.heartbeat_sent_inc();

        self.queues.heartbeat.0.send(heartbeat).await
    }

    pub async fn send_task(&mut self) -> Result<(), ()> {
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

        Ok(())
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
