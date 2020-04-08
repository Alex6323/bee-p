use crate::milestone::MilestoneIndex;

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
                            let target_milestone_index = *tangle().get_last_solid_milestone_index() + 1;

                            // match tangle().get_milestone_hash(target_milestone_index) {
                            //     Some(target_milestone_hash) => self.solidify(target_milestone_hash),
                            //     None => break
                            // TODO also request it
                            // }
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
