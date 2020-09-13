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

pub(crate) struct OtrsiYtrsiPropagatorWorkerEvent(pub(crate) Hash);
type OtrsiYtrsiPropagatorReceiver = ShutdownStream<Fuse<mpsc::UnboundedReceiver<OtrsiYtrsiPropagatorWorkerEvent>>>;

pub(crate) struct OtrsiYtrsiPropagatorWorker {
    receiver: OtrsiYtrsiPropagatorReceiver,
}

impl OtrsiYtrsiPropagatorWorker {
    pub(crate) fn new(receiver: OtrsiYtrsiPropagatorReceiver) -> Self {
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

        while let Some(OtrsiYtrsiPropagatorWorkerEvent(hash)) = self.receiver.next().await {
            self.propagate(hash);
        }

        info!("Stopped.");

        Ok(())
    }

}