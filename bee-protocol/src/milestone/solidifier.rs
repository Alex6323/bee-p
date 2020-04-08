use crate::{
    milestone::MilestoneIndex,
    protocol::Protocol,
};

use bee_bundle::Hash;
use bee_tangle::tangle;

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

pub(crate) struct MilestoneSolidifierWorkerEvent();

pub(crate) struct MilestoneSolidifierWorker {}

impl MilestoneSolidifierWorker {
    pub(crate) fn new() -> Self {
        Self {}
    }

    fn solidify(&self, hash: Hash) {
        // TODO Tangle traversal
    }

    pub(crate) async fn run(
        self,
        receiver: mpsc::Receiver<MilestoneSolidifierWorkerEvent>,
        shutdown: oneshot::Receiver<()>,
    ) {
        info!("[MilestoneSolidifierWorker ] Running.");

        let mut receiver_fused = receiver.fuse();
        let mut shutdown_fused = shutdown.fuse();

        loop {
            select! {
                event = receiver_fused.next() => {
                    if let Some(MilestoneSolidifierWorkerEvent()) = event {
                        // TODO impl Ord to avoid deref
                        while *tangle().get_last_solid_milestone_index() < *tangle().get_last_milestone_index() {
                            // TODO a bit cumbersome...
                            let target_milestone_index : bee_tangle::MilestoneIndex =
                                (*tangle().get_last_solid_milestone_index() + 1).into();

                            match tangle().get_milestone_hash(&target_milestone_index) {
                                Some(target_milestone_hash) => self.solidify(target_milestone_hash),
                                None => {
                                    // There is a gap, request the milestone
                                    Protocol::request_milestone(*target_milestone_index);
                                    break
                                }
                            }
                        }
                    }
                },
                _ = shutdown_fused => {
                    break;
                }
            }
        }

        info!("[MilestoneSolidifierWorker ] Stopped.");
    }
}

#[cfg(test)]
mod tests {}
