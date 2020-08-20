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

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};

use futures::{channel::mpsc, stream::StreamExt};
use log::info;

type Receiver = ShutdownStream<mpsc::Receiver<MilestoneSolidifierWorkerEvent>>;

pub(crate) struct MilestoneSolidifierWorkerEvent;

pub(crate) struct MilestoneSolidifierWorker {
    receiver: Receiver,
}

impl MilestoneSolidifierWorker {
    pub(crate) fn new(receiver: Receiver) -> Self {
        Self { receiver }
    }

    // async fn process_target(&self, target_index: u32) -> bool {
    //     match tangle().get_milestone_hash(target_index.into()) {
    //         Some(target_hash) => match self.solidify(target_hash, target_index).await {
    //             true => {
    //                 tangle().update_solid_milestone_index(target_index.into());
    //                 Protocol::broadcast_heartbeat(
    //                     *tangle().get_last_solid_milestone_index(),
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

    async fn solidify_milestone(&self) {
        let target_index = tangle().get_last_solid_milestone_index() + MilestoneIndex(1);

        // if let Some(target_hash) = tangle().get_milestone_hash(target_index) {
        //     if tangle().is_solid_transaction(&target_hash) {
        //         // TODO set confirmation index + trigger ledger
        //         tangle().update_last_solid_milestone_index(target_index);
        //         Protocol::broadcast_heartbeat(
        //             tangle().get_last_solid_milestone_index(),
        //             tangle().get_snapshot_milestone_index(),
        //         )
        //         .await;
        //     } else {
        //         Protocol::trigger_transaction_solidification(target_hash, target_index).await;
        //     }
        // }
        if let Some(target_hash) = tangle().get_milestone_hash(target_index) {
            if !tangle().is_solid_transaction(&target_hash) {
                Protocol::trigger_transaction_solidification(target_hash, target_index).await;
            }
        }
    }

    pub(crate) async fn run(mut self) -> Result<(), WorkerError> {
        info!("Running.");

        while let Some(MilestoneSolidifierWorkerEvent) = self.receiver.next().await {
            self.solidify_milestone().await;
            // while tangle().get_last_solid_milestone_index() < tangle().get_last_milestone_index() {
            //     if !self.process_target(*tangle().get_last_solid_milestone_index() + 1).await {
            //         break;
            //     }
            // }
        }

        info!("Stopped.");

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
