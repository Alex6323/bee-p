use crate::{milestone::MilestoneIndex, protocol::Protocol};

use bee_bundle::Hash;
use bee_tangle::tangle;

use std::collections::HashSet;

use futures::{
    channel::{mpsc, oneshot},
    future::FutureExt,
    select,
    stream::StreamExt,
};
use log::info;

pub(crate) struct TransactionSolidifierWorkerEvent(pub(crate) Hash, pub(crate) MilestoneIndex);

pub(crate) struct TransactionSolidifierWorker {}

impl TransactionSolidifierWorker {
    pub(crate) fn new() -> Self {
        Self {}
    }

    // TODO is the index even needed ? We request one milestone at a time ? No PriorityQueue ?

    async fn solidify(&self, hash: Hash, index: u32) -> bool {
        let mut missing_hashes = HashSet::new();

        tangle().walk_approvees_depth_first(
            hash,
            |_| {},
            |vertex| !vertex.is_solid(),
            |missing_hash| {
                missing_hashes.insert(*missing_hash);
            },
        );

        // TODO refactor with async closures when stabilized
        match missing_hashes.is_empty() {
            true => true,
            false => {
                for missing_hash in missing_hashes {
                    Protocol::request_transaction(missing_hash, index).await;
                }

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
                    if let Some(TransactionSolidifierWorkerEvent(hash, index)) = event {
                        self.solidify(hash, index).await;
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
