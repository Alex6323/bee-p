use crate::message::{
    Message,
    TransactionRequest,
};

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

// TODO use proper hash type
pub(crate) type TransactionRequesterWorkerEvent = [u8; 49];

pub(crate) struct TransactionRequesterWorker {
    receiver: mpsc::Receiver<TransactionRequesterWorkerEvent>,
    shutdown: oneshot::Receiver<()>,
}

impl TransactionRequesterWorker {
    pub(crate) fn new(
        receiver: mpsc::Receiver<TransactionRequesterWorkerEvent>,
        shutdown: oneshot::Receiver<()>,
    ) -> Self {
        Self { receiver, shutdown }
    }

    pub(crate) async fn run(self) {
        info!("[TransactionRequesterWorker ] Running.");

        let mut receiver_fused = self.receiver.fuse();
        let mut shutdown_fused = self.shutdown.fuse();

        loop {
            select! {
                hash = receiver_fused.next() => {
                    if let Some(hash) = hash {
                        let _bytes = TransactionRequest::new(hash).into_full_bytes();
                        // TODO we don't have any peer_id here
                    }
                },
                _ = shutdown_fused => {
                    break;
                }
            }
        }

        info!("[TransactionRequesterWorker ] Stopped.");
    }
}
