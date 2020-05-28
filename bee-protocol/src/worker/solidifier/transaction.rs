// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use crate::{
    milestone::{
        tangle::{tangle, Flags},
        MilestoneIndex,
    },
    protocol::Protocol,
};

use bee_tangle::traversal;
use bee_transaction::Hash;

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

    async fn solidify(&self, hash: Hash, index: MilestoneIndex) -> bool {
        let mut missing_hashes = HashSet::new();

        traversal::walk_approvees_dfs(
            &tangle().inner,
            hash,
            |_, v| v.get_meta().contains(Flags::SOLID),
            |_, _| {},
            |h| {
                missing_hashes.insert(*h);
            },
        );

        // TODO refactor with async closures when stabilized
        if missing_hashes.is_empty() {
            true
        } else {
            for missing_hash in missing_hashes {
                Protocol::request_transaction(missing_hash, index).await;
            }

            false
        }
    }

    pub(crate) async fn run(
        self,
        receiver: mpsc::Receiver<TransactionSolidifierWorkerEvent>,
        shutdown: oneshot::Receiver<()>,
    ) {
        info!("Running.");

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

        info!("Stopped.");
    }
}

#[cfg(test)]
mod tests {}
