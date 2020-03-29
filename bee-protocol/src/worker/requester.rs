use crate::{
    message::{
        Message,
        MilestoneRequest,
        TransactionRequest,
    },
    milestone::MilestoneIndex,
};

use futures::{
    channel::mpsc::Receiver,
    stream::StreamExt,
};
use log::info;

// Transaction requester worker

pub(crate) enum TransactionRequesterWorkerEvent {
    Request([u8; 49]),
}

pub(crate) struct TransactionRequesterWorker {
    receiver: Receiver<TransactionRequesterWorkerEvent>,
}

impl TransactionRequesterWorker {
    pub(crate) fn new(receiver: Receiver<TransactionRequesterWorkerEvent>) -> Self {
        Self { receiver: receiver }
    }

    pub(crate) async fn run(mut self) {
        info!("[TransactionRequesterWorker ] Running.");

        while let Some(event) = self.receiver.next().await {
            let _bytes = match event {
                TransactionRequesterWorkerEvent::Request(hash) => TransactionRequest::new(hash).into_full_bytes(),
            };
            // TODO we don't have any peer_id here
        }
    }
}

// Milestone requester worker

pub(crate) enum MilestoneRequesterWorkerEvent {
    Request(MilestoneIndex),
}

pub(crate) struct MilestoneRequesterWorker {
    receiver: Receiver<MilestoneRequesterWorkerEvent>,
}

impl MilestoneRequesterWorker {
    pub(crate) fn new(receiver: Receiver<MilestoneRequesterWorkerEvent>) -> Self {
        Self { receiver: receiver }
    }

    pub(crate) async fn run(mut self) {
        info!("[MilestoneRequesterWorker ] Running.");

        while let Some(event) = self.receiver.next().await {
            let _bytes = match event {
                MilestoneRequesterWorkerEvent::Request(index) => MilestoneRequest::new(index).into_full_bytes(),
            };
            // TODO we don't have any peer_id here
        }
    }
}
