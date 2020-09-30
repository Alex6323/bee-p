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
    tangle::tangle,
    worker::tip_candidate_validator::{BundleInfo, TipCandidateValidatorWorker, TipCandidateValidatorWorkerEvent},
    Milestone, MilestoneIndex,
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{node::Node, worker::Worker};
use bee_crypto::ternary::Hash;
use bee_tangle::helper::load_bundle_builder;
use bee_transaction::Vertex;

use async_trait::async_trait;
use futures::{channel::mpsc, stream::StreamExt};
use log::{error, info, warn};

use crate::worker::{TrsiPropagatorWorker, TrsiPropagatorWorkerEvent};
use std::{any::TypeId, collections::HashSet};

pub(crate) struct MilestoneConeUpdaterWorkerEvent(pub(crate) Milestone);

pub(crate) struct MilestoneConeUpdaterWorker {
    pub(crate) tx: mpsc::UnboundedSender<MilestoneConeUpdaterWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for MilestoneConeUpdaterWorker {
    type Config = ();
    type Error = WorkerError;

    fn dependencies() -> &'static [TypeId] {
        Box::leak(Box::from(vec![TypeId::of::<TrsiPropagatorWorker>()]))
    }

    async fn start(node: &N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded();
        let trsi_propagator = node.worker::<TrsiPropagatorWorker>().unwrap().tx.clone();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx);

            while let Some(MilestoneConeUpdaterWorkerEvent(milestone)) = receiver.next().await {
                let children = update_transactions_referenced_by_milestone(milestone.hash, milestone.index);
                // Propagate updated OTRSI/YTRSI values to all direct/indirect approvers
                if let Err(e) = trsi_propagator.unbounded_send(
                    TrsiPropagatorWorkerEvent::UpdateTransactionsReferencedByMilestone(children),
                ) {
                    error!("Failed to send hash to TRSI propagator: {:?}.", e);
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}

// When a new milestone gets solid, OTRSI and YTRSI of all transactions that belong to the given cone must be
// updated. This function returns the children of all updated transactions.
fn update_transactions_referenced_by_milestone(tail_hash: Hash, milestone_index: MilestoneIndex) -> HashSet<Hash> {
    info!("Updating transactions referenced by milestone {}.", *milestone_index);
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

    children
}
