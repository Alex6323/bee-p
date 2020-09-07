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
use bee_tangle::traversal;

use futures::{
    channel::mpsc,
    stream::{Fuse, StreamExt},
};
use log::info;

pub(crate) const MILESTONE_COUNT: u32 = 5;

type Receiver = ShutdownStream<Fuse<mpsc::UnboundedReceiver<MilestoneSolidifierWorkerEvent>>>;

pub(crate) struct MilestoneSolidifierWorkerEvent(pub MilestoneIndex);

pub(crate) struct MilestoneSolidifierWorker {
    receiver: Receiver,
}

impl MilestoneSolidifierWorker {
    pub(crate) fn new(receiver: Receiver) -> Self {
        Self { receiver }
    }

    fn solidify_milestone(&self, target_index: MilestoneIndex) {
        if let Some(target_hash) = tangle().get_milestone_hash(target_index) {
            if !tangle().is_solid_transaction(&target_hash) {
                traversal::visit_parents_depth_first(
                    tangle(),
                    target_hash,
                    |hash, _, metadata| {
                        !metadata.flags.is_solid() && !Protocol::get().requested_transactions.contains_key(&hash)
                    },
                    |_, _, _| {},
                    |missing_hash| {
                        if !tangle().is_solid_entry_point(missing_hash)
                            && !Protocol::get().requested_transactions.contains_key(&missing_hash)
                        {
                            Protocol::request_transaction(*missing_hash, target_index);
                        }
                    },
                );
            }
        }
    }

    pub(crate) async fn run(mut self) -> Result<(), WorkerError> {
        info!("Running.");

        while let Some(MilestoneSolidifierWorkerEvent(index)) = self.receiver.next().await {
            self.solidify_milestone(index);
        }

        info!("Stopped.");

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
