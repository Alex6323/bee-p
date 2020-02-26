use crate::message::{
    Handshake, Heartbeat, LegacyGossip, MilestoneRequest, TransactionBroadcast, TransactionRequest,
};
use crate::neighbor::Neighbor;
use crate::node::NodeMetrics;

use async_std::task::spawn;
use futures::channel::mpsc::SendError;
use futures::sink::SinkExt;

struct Node {
    metrics: NodeMetrics,
}

impl Node {
    async fn actor() {}

    fn start() {
        spawn(Self::actor());
    }

    pub async fn send_handshake(
        &self,
        neighbor: &mut Neighbor,
        handshake: Handshake,
    ) -> Result<(), SendError> {
        let res = neighbor.queues.handshake.0.send(handshake).await;

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
        let res = neighbor.queues.legacy_gossip.0.send(legacy_gossip).await;

        if res.is_ok() {
            neighbor.metrics.legacy_gossip_sent_inc();
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
            .queues
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
            .queues
            .transaction_broadcast
            .0
            .send(transaction_broadcast)
            .await;

        if res.is_ok() {
            neighbor.metrics.transaction_broadcast_sent_inc();
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
            .queues
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
        let res = neighbor.queues.heartbeat.0.send(heartbeat).await;

        if res.is_ok() {
            neighbor.metrics.heartbeat_sent_inc();
            self.metrics.heartbeat_sent_inc();
        }

        res
    }
}
