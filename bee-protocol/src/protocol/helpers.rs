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
        BroadcasterWorkerEvent,
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
    // MilestoneRequest

    pub fn request_milestone(index: MilestoneIndex, to_epid: Option<EndpointId>) {
        Protocol::get()
            .milestone_requester_worker
            .0
            .insert(MilestoneRequesterWorkerEntry(index, to_epid));
    }

    pub fn request_last_milestone(to_epid: Option<EndpointId>) {
        Protocol::request_milestone(0, to_epid);
    }

    pub fn milestone_requester_is_empty() -> bool {
        Protocol::get().milestone_requester_worker.0.is_empty()
    }

    // TransactionBroadcast

    pub async fn send_transaction(to_epid: EndpointId, transaction: &[u8]) {
        SenderWorker::<TransactionBroadcast>::send(&to_epid, TransactionBroadcast::new(transaction)).await;
    }

    // This doesn't use `send_transaction` because answering a request and broadcasting are different priorities
    pub(crate) async fn broadcast_transaction_message(
        from_epid: Option<EndpointId>,
        transaction_broadcast: TransactionBroadcast,
    ) {
        if let Err(e) = Protocol::get()
            .broadcaster_worker
            .0
            // TODO try to avoid
            .clone()
            .send(BroadcasterWorkerEvent(from_epid, transaction_broadcast))
            .await
        {
            warn!("[Protocol ] Broadcasting transaction failed: {}.", e);
        }
    }

    // This doesn't use `send_transaction` because answering a request and broadcasting are different priorities
    pub async fn broadcast_transaction(from_epid: Option<EndpointId>, transaction: &[u8]) {
        Protocol::broadcast_transaction_message(from_epid, TransactionBroadcast::new(transaction)).await;
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

    // Heartbeat

    pub async fn send_heartbeat(
        to_epid: EndpointId,
        first_solid_milestone_index: MilestoneIndex,
        last_solid_milestone_index: MilestoneIndex,
    ) {
        SenderWorker::<Heartbeat>::send(
            &to_epid,
            Heartbeat::new(first_solid_milestone_index, last_solid_milestone_index),
        )
        .await;
    }

    pub async fn broadcast_heartbeat(
        first_solid_milestone_index: MilestoneIndex,
        last_solid_milestone_index: MilestoneIndex,
    ) {
        for entry in Protocol::get().contexts.iter() {
            Protocol::send_heartbeat(*entry.key(), first_solid_milestone_index, last_solid_milestone_index).await;
        }
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
