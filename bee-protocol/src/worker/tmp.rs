use crate::protocol::Protocol;

use bee_bundle::Hash;
use bee_tangle::tangle;

use std::collections::HashSet;

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

pub(crate) struct TransactionSolidifierWorkerEvent(pub(crate) Hash);

pub(crate) struct TransactionSolidifierWorker {}

impl TransactionSolidifierWorker {
    pub(crate) fn new() -> Self {
        Self {}
    }

    async fn solidify(&self, hash: Hash, target_index: u32) -> bool {
        let mut missing_hashes = HashSet::new();

        tangle().walk_approvees_depth_first(
            hash,
            |_| {},
            |transaction| true,
            |missing_hash| {
                missing_hashes.insert(*missing_hash);
            },
        );

        // TODO refactor with async closures when stabilized
        match missing_hashes.is_empty() {
            true => true,
            false => {
                for missing_hash in missing_hashes {
                    Protocol::request_transaction(missing_hash, target_index).await;
                }

                false
            }
        }
    }

    async fn process_target(&self, target_index: u32) -> bool {
        match tangle().get_milestone_hash(target_index.into()) {
            Some(target_hash) => match self.solidify(target_hash, target_index).await {
                true => {
                    tangle().update_solid_milestone_index(target_index.into());
                    Protocol::broadcast_heartbeat(
                        *tangle().get_solid_milestone_index(),
                        *tangle().get_snapshot_milestone_index(),
                    )
                    .await;
                    true
                }
                false => false,
            },
            None => {
                // There is a gap, request the milestone
                Protocol::request_milestone(target_index, None);
                false
            }
        }
    }

    pub(crate) async fn run(
        self,
        receiver: mpsc::Receiver<TransactionSolidifierWorkerEvent>,
        shutdown: oneshot::Receiver<()>,
    ) {
        info!("[TransactionSolidifierWorker ] Running.");

        let mut receiver_fused = receiver.fuse();
        let mut shutdown_fused = shutdown.fuse();

        loop {
            select! {
                event = receiver_fused.next() => {
                    if let Some(TransactionSolidifierWorkerEvent(hash)) = event {
                        while tangle().get_solid_milestone_index() < tangle().get_last_milestone_index() {
                            if !self.process_target(*tangle().get_solid_milestone_index() + 1).await {
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

        info!("[TransactionSolidifierWorker ] Stopped.");
    }
}

#[cfg(test)]
mod tests {}
