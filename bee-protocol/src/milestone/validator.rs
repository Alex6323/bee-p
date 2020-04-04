use bee_bundle::Hash;

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

pub(crate) type MilestoneValidatorWorkerEvent = Hash;

pub(crate) struct MilestoneValidatorWorker {
    receiver: mpsc::Receiver<MilestoneValidatorWorkerEvent>,
    shutdown: oneshot::Receiver<()>,
}

impl MilestoneValidatorWorker {
    pub(crate) fn new(
        receiver: mpsc::Receiver<MilestoneValidatorWorkerEvent>,
        shutdown: oneshot::Receiver<()>,
    ) -> Self {
        Self { receiver, shutdown }
    }

    pub(crate) async fn run(self) {
        info!("[MilestoneValidatorWorker ] Running.");

        let mut receiver_fused = self.receiver.fuse();
        let mut shutdown_fused = self.shutdown.fuse();

        loop {
            select! {
                _hash = receiver_fused.next() => {
                    if let Some(_hash) = _hash {
                    }
                },
                _ = shutdown_fused => {
                    break;
                }
            }
        }

        info!("[MilestoneValidatorWorker ] Stopped.");
    }
}
