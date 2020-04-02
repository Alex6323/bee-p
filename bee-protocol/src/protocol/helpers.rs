use crate::{
    message::{
        Heartbeat,
        MilestoneRequest,
        TransactionBroadcast,
        TransactionRequest,
    },
    milestone::MilestoneIndex,
    protocol::Protocol,
    worker::SenderWorker,
};

use bee_network::EndpointId;

use futures::sink::SinkExt;
use log::warn;

impl Protocol {
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
        SenderWorker::<Heartbeat>::broadcast(Heartbeat::new(first_solid_milestone_index, last_solid_milestone_index))
            .await;
    }

    pub async fn send_milestone_request(epid: EndpointId, index: MilestoneIndex) {
        SenderWorker::<MilestoneRequest>::send(&epid, MilestoneRequest::new(index)).await;
    }

    pub async fn broadcast_milestone_request(index: MilestoneIndex) {
        SenderWorker::<MilestoneRequest>::broadcast(MilestoneRequest::new(index)).await;
    }

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

    //  TODO constant

    pub async fn send_transaction_request(epid: EndpointId, hash: [u8; 49]) {
        SenderWorker::<TransactionRequest>::send(&epid, TransactionRequest::new(hash)).await;
    }

    pub async fn broadcast_transaction_request(hash: [u8; 49]) {
        SenderWorker::<TransactionRequest>::broadcast(TransactionRequest::new(hash)).await;
    }
}
