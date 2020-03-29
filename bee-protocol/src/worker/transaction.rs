use crate::message::TransactionBroadcast;

use futures::{
    channel::mpsc::Receiver,
    stream::StreamExt,
};
use log::info;

pub(crate) type TransactionWorkerEvent = TransactionBroadcast;

pub(crate) struct TransactionWorker {
    receiver: Receiver<TransactionWorkerEvent>,
}

impl TransactionWorker {
    pub(crate) fn new(receiver: Receiver<TransactionWorkerEvent>) -> Self {
        Self { receiver: receiver }
    }

    pub(crate) async fn run(mut self) {
        info!("[TransactionWorker ] Running.");

        while let Some(transaction) = self.receiver.next().await {}
    }
}
