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

pub(crate) enum RequesterWorkerEvent {
    TransactionRequest([u8; 49]),
    MilestoneRequest(MilestoneIndex),
}

pub(crate) struct RequesterWorker {
    receiver: Receiver<RequesterWorkerEvent>,
}

impl RequesterWorker {
    pub(crate) fn new(receiver: Receiver<RequesterWorkerEvent>) -> Self {
        Self { receiver: receiver }
    }

    pub(crate) async fn run(mut self) {
        info!("[RequesterWorker ] Running.");

        while let Some(event) = self.receiver.next().await {
            let _bytes = match event {
                RequesterWorkerEvent::TransactionRequest(hash) => TransactionRequest::new(hash).into_full_bytes(),
                RequesterWorkerEvent::MilestoneRequest(index) => MilestoneRequest::new(index).into_full_bytes(),
            };
            // TODO we don't have any peer_id here
        }
    }
}
