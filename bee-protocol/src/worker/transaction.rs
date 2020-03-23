use crate::message::TransactionBroadcast;

use futures::channel::mpsc::Receiver;
use futures::stream::StreamExt;
use log::info;

pub enum TransactionWorkerEvent {
    Transaction(TransactionBroadcast),
}

pub struct TransactionWorker {
    receiver: Receiver<TransactionWorkerEvent>,
}

impl TransactionWorker {
    pub fn new(receiver: Receiver<TransactionWorkerEvent>) -> Self {
        Self { receiver: receiver }
    }

    pub async fn run(mut self) {
        info!("[TransactionWorker ] Running.");

        while let Some(TransactionWorkerEvent::Transaction(transaction)) = self.receiver.next().await {}
    }
}
