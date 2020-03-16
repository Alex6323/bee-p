use crate::message::TransactionBroadcast;

use futures::channel::mpsc::Receiver;
use futures::stream::StreamExt;

pub(crate) struct TransactionWorker {
    receiver: Receiver<TransactionBroadcast>,
}

impl TransactionWorker {
    pub(crate) fn new(receiver: Receiver<TransactionBroadcast>) -> Self {
        Self { receiver: receiver }
    }

    pub(crate) async fn run(mut self) {
        while let Some(TransactionBroadcast { transaction }) = self.receiver.next().await {}
    }
}
