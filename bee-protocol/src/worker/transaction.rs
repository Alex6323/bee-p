use crate::message::TransactionBroadcast;

use futures::{
    channel::mpsc::Receiver,
    stream::StreamExt,
};
use log::info;

pub(crate) enum TransactionWorkerEvent {
    Transaction(TransactionBroadcast),
}

pub(crate) struct TransactionWorker {
    receiver: Receiver<TransactionWorkerEvent>,
}

impl TransactionWorker {
    pub(crate) fn new(receiver: Receiver<TransactionWorkerEvent>) -> Self {
        Self { receiver: receiver }
    }

    pub(crate) async fn run(mut self) {
        info!("[TransactionWorker ] Running.");

        while let Some(TransactionWorkerEvent::Transaction(transaction)) = self.receiver.next().await {}
    }
}
