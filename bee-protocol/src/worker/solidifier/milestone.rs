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

use crate::{milestone::MilestoneIndex, protocol::Protocol, tangle::tangle};

use bee_common::worker::Error as WorkerError;

use futures::{
    channel::{mpsc, oneshot},
    future::FutureExt,
    select,
    stream::StreamExt,
};
use log::info;

const MILESTONE_REQUEST_RANGE: u8 = 50;

pub(crate) struct MilestoneSolidifierWorkerEvent();

pub(crate) struct MilestoneSolidifierWorker {}

impl MilestoneSolidifierWorker {
    pub(crate) fn new() -> Self {
        Self {}
    }

    // async fn solidify(&self, hash: Hash, target_index: u32) -> bool {
    //     let mut missing_hashes = HashSet::new();
    //
    //     tangle().walk_approvees_depth_first(
    //         hash,
    //         |_| {},
    //         |transaction| true,
    //         |missing_hash| {
    //             missing_hashes.insert(*missing_hash);
    //         },
    //     );
    //
    //     // TODO refactor with async closures when stabilized
    //     match missing_hashes.is_empty() {
    //         true => true,
    //         false => {
    //             for missing_hash in missing_hashes {
    //                 Protocol::request_transaction(missing_hash, target_index).await;
    //             }
    //
    //             false
    //         }
    //     }
    // }
    //
    // async fn process_target(&self, target_index: u32) -> bool {
    //     match tangle().get_milestone_hash(target_index.into()) {
    //         Some(target_hash) => match self.solidify(target_hash, target_index).await {
    //             true => {
    //                 tangle().update_solid_milestone_index(target_index.into());
    //                 Protocol::broadcast_heartbeat(
    //                     *tangle().get_solid_milestone_index(),
    //                     *tangle().get_snapshot_milestone_index(),
    //                 )
    //                 .await;
    //                 true
    //             }
    //             false => false,
    //         },
    //         None => {
    //             // There is a gap, request the milestone
    //             Protocol::request_milestone(target_index, None);
    //             false
    //         }
    //     }
    // }

    fn request_milestones(&self) {
        let solid_milestone_index = *tangle().get_solid_milestone_index();

        // TODO this may request unpublished milestones
        for index in solid_milestone_index..solid_milestone_index + MILESTONE_REQUEST_RANGE as u32 {
            let index = index.into();
            if !tangle().contains_milestone(index) {
                Protocol::request_milestone(index, None);
            }
        }
    }

    async fn solidify_milestone(&self) {
        let target_index = tangle().get_solid_milestone_index() + MilestoneIndex(1);

        if let Some(target_hash) = tangle().get_milestone_hash(target_index.into()) {
            if tangle().is_solid_transaction(&target_hash) {
                // TODO set confirmation index + trigger ledger
                tangle().update_solid_milestone_index(target_index.into());
                Protocol::broadcast_heartbeat(
                    tangle().get_solid_milestone_index(),
                    tangle().get_snapshot_milestone_index(),
                )
                .await;
            } else {
                Protocol::trigger_transaction_solidification(target_hash, target_index).await;
            }
        }
    }

    pub(crate) async fn run(
        self,
        receiver: mpsc::Receiver<MilestoneSolidifierWorkerEvent>,
        shutdown: oneshot::Receiver<()>,
    ) -> Result<(), WorkerError> {
        info!("Running.");

        let mut receiver_fused = receiver.fuse();
        let mut shutdown_fused = shutdown.fuse();

        loop {
            select! {
                event = receiver_fused.next() => {
                    if let Some(MilestoneSolidifierWorkerEvent()) = event {
                        self.request_milestones();
                        self.solidify_milestone().await;
                        // while tangle().get_solid_milestone_index() < tangle().get_last_milestone_index() {
                        //     if !self.process_target(*tangle().get_solid_milestone_index() + 1).await {
                        //         break;
                        //     }
                        // }
                    }
                },
                _ = shutdown_fused => {
                    break;
                }
            }
        }

        info!("Stopped.");

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
