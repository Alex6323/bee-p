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

pub(crate) struct TransactionWorker {}

impl TransactionWorker {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) async fn run(self, receiver: mpsc::Receiver<TransactionWorkerEvent>, shutdown: oneshot::Receiver<()>) {
        info!("[TransactionWorker ] Running.");

        let mut receiver_fused = receiver.fuse();
        let mut shutdown_fused = shutdown.fuse();

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
