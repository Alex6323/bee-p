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

pub(crate) struct TransactionRootSnapshotIndexPropagatorWorkerEvent(pub(crate) Hash);
type TransactionRootSnapshotIndexPropagatorReceiver = ShutdownStream<Fuse<mpsc::UnboundedReceiver<TransactionRootSnapshotIndexPropagatorWorkerEvent>>>;

pub(crate) struct TransactionRootSnapshotIndexPropagatorWorker {
    receiver: TransactionRootSnapshotIndexPropagatorReceiver,
}

impl TransactionRootSnapshotIndexPropagatorWorker {
    pub(crate) fn new(receiver: TransactionRootSnapshotIndexPropagatorReceiver) -> Self {
        Self { receiver }
    }

    fn propagate(&mut self, root: Hash) {
        let mut children = vec![root];
        while let Some(hash) = children.pop() {
            // get best otrsi and ytrsi from parents
            let tx = tangle().get(&hash).unwrap();
            let trunk_otsri = tangle().otrsi(tx.trunk());
            let branch_otsri = tangle().otrsi(tx.branch());
            let trunk_ytrsi = tangle().ytrsi(tx.trunk());
            let branch_ytrsi = tangle().ytrsi(tx.branch());

            if trunk_otsri.is_none() || branch_otsri.is_none() || trunk_ytrsi.is_none() || branch_ytrsi.is_none() {
                continue;
            }

            // check if already confirmed by update_transactions_referenced_by_milestone()
            if tangle().get_metadata(&hash).unwrap().cone_index.is_some() {
                continue;
            }

            // in case the transaction already inherited the best otrsi and ytrsi, continue
            let current_otrsi = tangle().get_metadata(&hash).unwrap().otrsi;
            let current_ytrsi = tangle().get_metadata(&hash).unwrap().ytrsi;
            let best_otrsi = max(trunk_otsri.unwrap(), branch_otsri.unwrap());
            let best_ytrsi = min(trunk_ytrsi.unwrap(), branch_ytrsi.unwrap());

            if current_otrsi.is_some()
                && current_ytrsi.is_some()
                && current_otrsi.unwrap() == best_otrsi
                && current_ytrsi.unwrap() == best_ytrsi
            {
                continue;
            }

            tangle().update_metadata(&hash, |metadata| {
                metadata.otrsi = Some(best_otrsi);
                metadata.ytrsi = Some(best_ytrsi);
            });

            // propagate otrsi and ytrsi to children
            for child in tangle().get_children(&hash) {
                children.push(child);
            }
        }
    }

    pub(crate) async fn run(mut self) -> Result<(), WorkerError> {
        info!("Running.");

        while let Some(TransactionRootSnapshotIndexPropagatorWorkerEvent(hash)) = self.receiver.next().await {
            self.propagate(hash);
        }

        info!("Stopped.");

        Ok(())
    }

}
