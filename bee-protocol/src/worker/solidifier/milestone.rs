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

use crate::{milestone::MilestoneIndex, protocol::Protocol, tangle::tangle, worker::TransactionSolidifierWorkerEvent};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_crypto::ternary::Hash;

use futures::{
    channel::mpsc,
    stream::{Fuse, StreamExt},
};
use log::info;

type Receiver = ShutdownStream<Fuse<mpsc::UnboundedReceiver<MilestoneSolidifierWorkerEvent>>>;

pub(crate) const TRANSACTION_SOLIDIFIER_COUNT: usize = 10;

pub(crate) enum MilestoneSolidifierWorkerEvent {
    ReceivedTransaction(Hash, MilestoneIndex),
    NewSolidMilestone(MilestoneIndex),
    Idle,
}

pub(crate) struct MilestoneSolidifierWorker {
    receiver: Receiver,
    senders: Vec<mpsc::UnboundedSender<TransactionSolidifierWorkerEvent>>,
    lowest_index: MilestoneIndex,
}

impl MilestoneSolidifierWorker {
    pub(crate) fn new(
        receiver: Receiver,
        senders: Vec<mpsc::UnboundedSender<TransactionSolidifierWorkerEvent>>,
    ) -> Self {
        assert_ne!(TRANSACTION_SOLIDIFIER_COUNT, 0);
        Self {
            receiver,
            senders,
            lowest_index: tangle().get_last_solid_milestone_index() + MilestoneIndex(1),
        }
    }

    fn trigger_transaction_solidification(
        sender: &mpsc::UnboundedSender<TransactionSolidifierWorkerEvent>,
        hash: Hash,
        index: MilestoneIndex,
    ) {
        if !tangle().is_solid_transaction(&hash) {
            sender.unbounded_send(TransactionSolidifierWorkerEvent(hash, index));
        }
    }

    pub(crate) async fn run(mut self) -> Result<(), WorkerError> {
        info!("Running.");

        while let Some(event) = self.receiver.next().await {
            match event {
                MilestoneSolidifierWorkerEvent::ReceivedTransaction(hash, index) => {
                    if index < self.lowest_index {
                        // We already solidified the milestone for this transaction
                    } else {
                        let sender_pos = (index.0 - self.lowest_index.0) as usize;
                        if let Some(sender) = self.senders.get(sender_pos) {
                            Self::trigger_transaction_solidification(sender, hash, index);
                        } else {
                            // This only happens if `index > self.lowest_index +
                            // TRANSACTION_SOLIDIFIER_COUNT`. Meaning that the transaction is too
                            // new.
                        }
                    }
                }
                MilestoneSolidifierWorkerEvent::NewSolidMilestone(index) => {
                    if index < self.lowest_index {
                        // We already were notified about this milestone.
                    } else if index == self.lowest_index {
                        // Update lowest milestone index.
                        self.lowest_index = self.lowest_index + MilestoneIndex(1);
                        // Compute the next target milestone index.
                        let target_index = self.lowest_index + MilestoneIndex(TRANSACTION_SOLIDIFIER_COUNT as u32);
                        // Trigger solidification if we already have the milestone's transaction.
                        if let Some(target_hash) = tangle().get_milestone_hash(target_index) {
                            Self::trigger_transaction_solidification(&self.senders[0], target_hash, target_index);
                        }
                    } else {
                        // We shouldn't be able to solidify any milestone that comes after
                        // `self.lowest_index`
                        panic!();
                    }
                }
                MilestoneSolidifierWorkerEvent::Idle => {
                    for (i, sender) in self.senders.iter().enumerate() {
                        let target_index = self.lowest_index + MilestoneIndex(i as u32);
                        if let Some(target_hash) = tangle().get_milestone_hash(target_index) {
                            Self::trigger_transaction_solidification(sender, target_hash, target_index);
                        }
                    }
                }
            }
        }

        info!("Stopped.");

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
