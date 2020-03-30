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

// TODO use proper hash type
pub(crate) type TransactionRequesterWorkerEvent = [u8; 49];

pub(crate) struct TransactionRequesterWorker {
    receiver: Receiver<TransactionRequesterWorkerEvent>,
}

impl TransactionRequesterWorker {
    pub(crate) fn new(receiver: Receiver<TransactionRequesterWorkerEvent>) -> Self {
        Self { receiver: receiver }
    }

    pub(crate) async fn run(mut self) {
        info!("[TransactionRequesterWorker ] Running.");

        while let Some(hash) = self.receiver.next().await {
            let _bytes = TransactionRequest::new(hash).into_full_bytes();
            // TODO we don't have any peer_id here
        }
    }
}

// Milestone requester worker

pub(crate) type MilestoneRequesterWorkerEvent = MilestoneIndex;

pub(crate) struct MilestoneRequesterWorker {
    receiver: Receiver<MilestoneRequesterWorkerEvent>,
}

impl MilestoneRequesterWorker {
    pub(crate) fn new(receiver: Receiver<MilestoneRequesterWorkerEvent>) -> Self {
        Self { receiver: receiver }
    }

    pub(crate) async fn run(mut self) {
        info!("[MilestoneRequesterWorker ] Running.");

        while let Some(index) = self.receiver.next().await {
            let _bytes = MilestoneRequest::new(index).into_full_bytes();
            // TODO we don't have any peer_id here
        }
    }
}
