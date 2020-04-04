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

pub(crate) struct MilestoneValidatorWorker {}

impl MilestoneValidatorWorker {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) async fn run(
        self,
        receiver: mpsc::Receiver<MilestoneValidatorWorkerEvent>,
        shutdown: oneshot::Receiver<()>,
    ) {
        info!("[MilestoneValidatorWorker ] Running.");

        let mut receiver_fused = receiver.fuse();
        let mut shutdown_fused = shutdown.fuse();

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
