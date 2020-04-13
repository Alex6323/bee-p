use crate::{
    message::{
        Heartbeat,
        TransactionBroadcast,
    },
    milestone::{
        MilestoneIndex,
        MilestoneSolidifierWorkerEvent,
    },
    protocol::Protocol,
    worker::{
        MilestoneRequesterWorkerEntry,
        SenderWorker,
        TransactionRequesterWorkerEntry,
    },
};

use bee_bundle::Hash;
use bee_network::EndpointId;

use futures::sink::SinkExt;
use log::warn;

impl Protocol {
    // Heartbeat

    pub async fn send_heartbeat(
        epid: EndpointId,
        first_solid_milestone_index: MilestoneIndex,
        last_solid_milestone_index: MilestoneIndex,
    ) {
        SenderWorker::<Heartbeat>::send(
            &epid,
            Heartbeat::new(first_solid_milestone_index, last_solid_milestone_index),
        )
        .await;
    }

    pub async fn broadcast_heartbeat(
        first_solid_milestone_index: MilestoneIndex,
        last_solid_milestone_index: MilestoneIndex,
    ) {
        for epid in Protocol::get().contexts.read().await.keys() {
            Protocol::send_heartbeat(*epid, first_solid_milestone_index, last_solid_milestone_index).await;
        }
    }

    // MilestoneRequest

    pub fn request_milestone(index: MilestoneIndex) {
        Protocol::get()
            .milestone_requester_worker
            .0
            .insert(MilestoneRequesterWorkerEntry(index));
    }

    pub fn request_latest_milestone() {
        Protocol::request_milestone(0);
    }

    pub fn milestone_requester_is_empty() -> bool {
        Protocol::get().milestone_requester_worker.0.is_empty()
    }

    // TransactionBroadcast

    pub async fn send_transaction(epid: EndpointId, transaction: &[u8]) {
        SenderWorker::<TransactionBroadcast>::send(&epid, TransactionBroadcast::new(transaction)).await;
    }

    // TODO explain why different
    pub async fn broadcast_transaction(transaction: &[u8]) {
        if let Err(e) = Protocol::get()
            .broadcaster_worker
            // TODO try to avoid
            .0
            .clone()
            .send(TransactionBroadcast::new(transaction))
            .await
        {
            warn!("[Protocol ] Broadcasting transaction failed: {}.", e);
        }
    }

    // TransactionRequest

    pub async fn request_transaction(hash: Hash, index: MilestoneIndex) {
        Protocol::get()
            .transaction_requester_worker
            .0
            .insert(TransactionRequesterWorkerEntry(hash, index));
    }

    pub fn transaction_requester_is_empty() -> bool {
        Protocol::get().transaction_requester_worker.0.is_empty()
    }

    // MilestoneSolidifier

    pub async fn trigger_milestone_solidification() {
        if let Err(e) = Protocol::get()
            .milestone_solidifier_worker
            // TODO try to avoid clone
            .0
            .clone()
            .send(MilestoneSolidifierWorkerEvent())
            .await
        {
            warn!("[Protocol ] Triggering milestone solidification failed: {}.", e);
        }
    }
}
