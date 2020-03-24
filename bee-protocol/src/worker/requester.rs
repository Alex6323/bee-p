use crate::message::{
    Message,
    MilestoneRequest,
    TransactionRequest,
};

use futures::channel::mpsc::Receiver;
use futures::stream::StreamExt;
use log::info;

pub enum RequesterWorkerEvent {
    TransactionRequest([u8; 49]),
    // TODO use MilestonIndex
    MilestoneRequest(u32),
}

pub struct RequesterWorker {
    receiver: Receiver<RequesterWorkerEvent>,
}

impl RequesterWorker {
    pub fn new(receiver: Receiver<RequesterWorkerEvent>) -> Self {
        Self { receiver: receiver }
    }

    pub async fn run(mut self) {
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
