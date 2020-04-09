use crate::protocol::Protocol;

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

    fn solidify(&self, hash: Hash) -> bool {
        // TODO Tangle traversal
        false
    }

    async fn process_target(&self, target_milestone_index: u32) -> bool {
        match tangle().get_milestone_hash(&(target_milestone_index.into())) {
            Some(target_milestone_hash) => match self.solidify(target_milestone_hash) {
                true => {
                    tangle().update_last_solid_milestone_index(target_milestone_index.into());
                    Protocol::broadcast_heartbeat(
                        *tangle().get_first_solid_milestone_index(),
                        *tangle().get_last_solid_milestone_index(),
                    )
                    .await;
                    true
                }
                false => false,
            },
            None => {
                // There is a gap, request the milestone
                Protocol::request_milestone(target_milestone_index);
                false
            }
        }
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
                        while tangle().get_last_solid_milestone_index() < tangle().get_last_milestone_index() {
                            if !self.process_target(*tangle().get_last_solid_milestone_index() + 1).await {
                                break;
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
