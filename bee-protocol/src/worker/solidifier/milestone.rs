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

type Receiver = ShutdownStream<mpsc::UnboundedReceiver<MilestoneSolidifierWorkerEvent>>;

pub(crate) struct MilestoneSolidifierWorkerEvent;

pub(crate) struct MilestoneSolidifierWorker {
    receiver: Receiver,
}

impl MilestoneSolidifierWorker {
    pub(crate) fn new(receiver: Receiver) -> Self {
        Self { receiver }
    }

    fn solidify_milestone(&self) {
        let target_index = tangle().get_last_solid_milestone_index() + MilestoneIndex(1);

        if let Some(target_hash) = tangle().get_milestone_hash(target_index) {
            if !tangle().is_solid_transaction(&target_hash) {
                Protocol::trigger_transaction_solidification(target_hash, target_index);
            }
        }
    }

    pub(crate) async fn run(mut self) -> Result<(), WorkerError> {
        info!("Running.");

        while let Some(MilestoneSolidifierWorkerEvent) = self.receiver.next().await {
            self.solidify_milestone();
        }

        info!("Stopped.");

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
