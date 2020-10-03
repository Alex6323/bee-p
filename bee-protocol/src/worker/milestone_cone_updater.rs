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

use crate::{tangle::tangle, Milestone, MilestoneIndex};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{node::Node, worker::Worker};
use bee_crypto::ternary::Hash;
use bee_tangle::helper::load_bundle_builder;
use bee_transaction::Vertex;

use async_trait::async_trait;
use futures::{channel::mpsc, stream::StreamExt};
use log::{error, info, warn};

use crate::worker::{PropagatorWorker, PropagatorWorkerEvent};
use std::{
    any::TypeId,
    cmp::{max, min},
    collections::HashSet,
};

pub(crate) struct MilestoneConeUpdaterWorkerEvent(pub(crate) Milestone);

pub(crate) struct MilestoneConeUpdaterWorker {
    pub(crate) tx: flume::Sender<MilestoneConeUpdaterWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for MilestoneConeUpdaterWorker {
    type Config = ();
    type Error = WorkerError;

    async fn start(node: &N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            while let Some(MilestoneConeUpdaterWorkerEvent(milestone)) = receiver.next().await {
                // When a new milestone gets solid, OTRSI and YTRSI of all transactions that belong to the given cone must be
                // updated. Furthermore, updated values will be propagated to the future.
                update_transactions_referenced_by_milestone(milestone.hash, milestone.index);
                // Update tip pool after all values got updated.
                tangle().update_tip_pool();
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}


fn update_transactions_referenced_by_milestone(tail_hash: Hash, milestone_index: MilestoneIndex) {

    let mut to_visit = vec![tail_hash];
    let mut visited = HashSet::new();
    let mut children = HashSet::new();

    while let Some(hash) = to_visit.pop() {
        if visited.contains(&hash) {
            continue;
        } else {
            visited.insert(hash.clone());
        }

        if tangle().is_solid_entry_point(&hash) {
            continue;
        }

        if tangle().get_metadata(&hash).unwrap().cone_index().is_some() {
            continue;
        }

        tangle().update_metadata(&hash, |metadata| {
            metadata.set_cone_index(milestone_index);
            metadata.set_otrsi(milestone_index);
            metadata.set_ytrsi(milestone_index);
        });

        for child in tangle().get_children(&hash) {
            children.insert(child);
        }

        let tx_ref = tangle().get(&hash).unwrap();
        to_visit.push(tx_ref.trunk().clone());
        to_visit.push(tx_ref.branch().clone());
    }

    update_children(children.clone().into_iter().collect());
}

fn update_children(mut children: Vec<Hash>) {
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

        // skip if transaction was already updated
        if tangle().get_metadata(&hash).unwrap().cone_index().is_some() {
            continue;
        }

        // in case the transaction already inherited the best otrsi and ytrsi, continue
        let current_otrsi = tangle().get_metadata(&hash).unwrap().otrsi();
        let current_ytrsi = tangle().get_metadata(&hash).unwrap().ytrsi();
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
            metadata.set_otrsi(best_otrsi);
            metadata.set_ytrsi(best_ytrsi);
        });

        // propagate to children
        for child in tangle().get_children(&hash) {
            children.push(child);
        }
    }
}
