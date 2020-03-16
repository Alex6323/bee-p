use crate::message::{LegacyGossip, TransactionBroadcast};

use futures::channel::mpsc::Receiver;
use futures::stream::StreamExt;

pub(crate) enum TransactionWorkerEvent {
    LegacyGossip(LegacyGossip),
    TransactionBroadcast(TransactionBroadcast),
}

pub(crate) struct TransactionWorker {
    receiver: Receiver<TransactionWorkerEvent>,
}

impl TransactionWorker {
    pub(crate) fn new(receiver: Receiver<TransactionWorkerEvent>) -> Self {
        Self { receiver: receiver }
    }

    pub(crate) async fn run(mut self) {
        while let Some(event) = self.receiver.next().await {
            match event {
                TransactionWorkerEvent::LegacyGossip(message) => {}
                TransactionWorkerEvent::TransactionBroadcast(message) => {}
            }
        }
    }
}
