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

use crate::{milestone::MilestoneIndex, protocol::Protocol, tangle::tangle, worker::Worker};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_tangle::traversal;

use async_trait::async_trait;
use futures::{
    channel::{mpsc, oneshot},
    stream::Fuse,
    StreamExt,
};
use log::{debug, info};

use std::any::TypeId;

pub(crate) struct MilestoneSolidifierWorkerEvent(pub MilestoneIndex);

pub(crate) struct MilestoneSolidifierWorker {
    queue: Vec<MilestoneIndex>,
    next_ms_index: MilestoneIndex,
}

#[async_trait]
impl Worker for MilestoneSolidifierWorker {
    const DEPS: &'static [TypeId] = &[];

    type Event = MilestoneSolidifierWorkerEvent;
    type Receiver = ShutdownStream<Fuse<mpsc::UnboundedReceiver<MilestoneSolidifierWorkerEvent>>>;

    async fn run(mut self, mut receiver: Self::Receiver) -> Result<(), WorkerError> {
        info!("Running.");

        while let Some(MilestoneSolidifierWorkerEvent(index)) = receiver.next().await {
            self.save_index(index);
            while let Some(index) = self.queue.pop() {
                if index == self.next_ms_index {
                    self.trigger_solidification_unchecked(index);
                } else {
                    self.queue.push(index);
                    break;
                }
            }
        }

        info!("Stopped.");

        Ok(())
    }
}

impl MilestoneSolidifierWorker {
    pub(crate) async fn new(next_ms_index: oneshot::Receiver<MilestoneIndex>) -> Self {
        Self {
            queue: vec![],
            next_ms_index: next_ms_index.await.unwrap(),
        }
    }

    fn trigger_solidification_unchecked(&mut self, target_index: MilestoneIndex) {
        if let Some(target_hash) = tangle().get_milestone_hash(target_index) {
            if !tangle().is_solid_transaction(&target_hash) {
                debug!("Triggered solidification for milestone {}.", *target_index);
                traversal::visit_parents_depth_first(
                    tangle(),
                    target_hash,
                    |hash, _, metadata| {
                        (!metadata.flags.is_requested() || *hash == target_hash)
                            && !metadata.flags.is_solid()
                            && !Protocol::get().requested_transactions.contains_key(&hash)
                    },
                    |_, _, _| {},
                    |_, _, _| {},
                    |missing_hash| Protocol::request_transaction(*missing_hash, target_index),
                );

                self.next_ms_index = target_index + MilestoneIndex(1);
            }
        }
    }

    fn save_index(&mut self, target_index: MilestoneIndex) {
        debug!("Storing milestone {}.", *target_index);
        if let Err(pos) = self.queue.binary_search_by(|index| target_index.cmp(index)) {
            self.queue.insert(pos, target_index);
        }
    }
}
