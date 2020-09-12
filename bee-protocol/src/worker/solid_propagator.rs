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
    event::{LatestSolidMilestoneChanged, TransactionSolidified},
    milestone::Milestone,
    protocol::Protocol,
    tangle::tangle,
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_crypto::ternary::Hash;
use bee_transaction::Vertex;

use futures::{
    channel::mpsc,
    stream::{Fuse, StreamExt},
};
use log::info;

use std::time::{SystemTime, UNIX_EPOCH};
use std::cmp::{max, min};

type SolidPropagatorReceiver = ShutdownStream<Fuse<mpsc::UnboundedReceiver<SolidPropagatorWorkerEvent>>>;
pub(crate) struct SolidPropagatorWorkerEvent(pub(crate) Hash);

pub(crate) struct SolidPropagatorWorker {
    receiver: SolidPropagatorReceiver,
}

impl SolidPropagatorWorker {
    pub(crate) fn new(receiver: SolidPropagatorReceiver) -> Self {
        Self { receiver }
    }

    fn propagate(&mut self, root: Hash) {
        let mut children = vec![root];

        while let Some(ref hash) = children.pop() {
            if tangle().is_solid_transaction(hash) {
                continue;
            }

            if let Some(tx) = tangle().get(&hash) {
                let mut index = None;

                if tangle().is_solid_transaction(tx.trunk()) && tangle().is_solid_transaction(tx.branch()) {
                    tangle().update_metadata(&hash, |metadata| {
                        metadata.flags.set_solid(true);
                        // This is possibly not sufficient as there is no guarantee a milestone has been validated
                        // before being solidified, we then also need to check when a milestone gets validated if it's
                        // already solid.
                        if metadata.flags.is_milestone() {
                            index = Some(metadata.milestone_index);
                        }
                        metadata.solidification_timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("Clock may have gone backwards")
                            .as_millis() as u64;
                    });

                    for child in tangle().get_children(&hash) {
                        children.push(child);
                    }

                    Protocol::get().bus.dispatch(TransactionSolidified(*hash));
                }

                if let Some(index) = index {
                    Protocol::get()
                        .bus
                        .dispatch(LatestSolidMilestoneChanged(Milestone { hash: *hash, index }));
                }
            }
        }
    }

    pub(crate) async fn run(mut self) -> Result<(), WorkerError> {
        info!("Running.");

        while let Some(SolidPropagatorWorkerEvent(hash)) = self.receiver.next().await {
            self.propagate(hash);
        }

        info!("Stopped.");

        Ok(())
    }

}
