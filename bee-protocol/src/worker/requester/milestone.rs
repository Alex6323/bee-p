use crate::{
    milestone::MilestoneIndex,
    protocol::Protocol,
};

use std::cmp::Ordering;

use futures::{
    channel::oneshot,
    future::FutureExt,
    select,
};
use log::info;

#[derive(Eq, PartialEq)]
pub(crate) struct MilestoneRequesterWorkerEntry(MilestoneIndex);

// TODO check that this is the right order
impl PartialOrd for MilestoneRequesterWorkerEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for MilestoneRequesterWorkerEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

pub(crate) struct MilestoneRequesterWorker {
    shutdown: oneshot::Receiver<()>,
}

impl MilestoneRequesterWorker {
    pub(crate) fn new(shutdown: oneshot::Receiver<()>) -> Self {
        Self { shutdown }
    }

    pub(crate) async fn run(self) {
        info!("[MilestoneRequesterWorker ] Running.");

        let mut shutdown_fused = self.shutdown.fuse();

        loop {
            select! {
                // TODO impl fused stream
                entry = Protocol::get().milestone_requester_worker.pop().fuse() => {
                    if let MilestoneRequesterWorkerEntry(index) = entry {
                //         let _bytes = MilestoneRequest::new(index).into_full_bytes();
                //         // TODO we don't have any peer_id here
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
