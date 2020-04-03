use crate::{
    message::{
        Message,
        MilestoneRequest,
    },
    milestone::MilestoneIndex,
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

pub(crate) type MilestoneRequesterWorkerEvent = MilestoneIndex;

pub(crate) struct MilestoneRequesterWorker {
    receiver: mpsc::Receiver<MilestoneRequesterWorkerEvent>,
    shutdown: oneshot::Receiver<()>,
}

impl MilestoneRequesterWorker {
    pub(crate) fn new(
        receiver: mpsc::Receiver<MilestoneRequesterWorkerEvent>,
        shutdown: oneshot::Receiver<()>,
    ) -> Self {
        Self { receiver, shutdown }
    }

    pub(crate) async fn run(self) {
        info!("[MilestoneRequesterWorker ] Running.");

        let mut receiver_fused = self.receiver.fuse();
        let mut shutdown_fused = self.shutdown.fuse();

        loop {
            select! {
                index = receiver_fused.next() => {
                    if let Some(index) = index {
                        let _bytes = MilestoneRequest::new(index).into_full_bytes();
                        // TODO we don't have any peer_id here
                    }
                },
                _ = shutdown_fused => {
                    break;
                }
            }
        }

        info!("[MilestoneRequesterWorker ] Stopped.");
    }
}
