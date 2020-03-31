use crate::message::TransactionBroadcast;

use futures::{
    channel::{
        mpsc,
        oneshot,
    },
    future::FutureExt,
    select,
    stream::StreamExt,
};
use log::info;

pub(crate) type TransactionWorkerEvent = TransactionBroadcast;

pub(crate) struct TransactionWorker {
    receiver: mpsc::Receiver<TransactionWorkerEvent>,
    shutdown: oneshot::Receiver<()>,
}

impl TransactionWorker {
    pub(crate) fn new(receiver: mpsc::Receiver<TransactionWorkerEvent>, shutdown: oneshot::Receiver<()>) -> Self {
        Self { receiver, shutdown }
    }

    pub(crate) async fn run(self) {
        info!("[TransactionWorker ] Running.");

        let mut receiver_fused = self.receiver.fuse();
        let mut shutdown_fused = self.shutdown.fuse();

        loop {
            select! {
                transaction = receiver_fused.next() => {
                    if let Some(transaction) = transaction {}
                },
                _ = shutdown_fused => {
                    break;
                }
            }
        }

        info!("[TransactionWorker ] Stopped.");
    }
}
