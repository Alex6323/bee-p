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
    worker::{TipCandidateValidatorWorker, TipCandidateValidatorWorkerEvent},
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{node::Node, worker::Worker};
use bee_crypto::ternary::Hash;
use bee_transaction::Vertex;

use async_trait::async_trait;
use futures::{channel::mpsc, stream::StreamExt};
use log::{info, warn};

use std::{
    any::TypeId,
    cmp::{max, min},
    collections::HashSet,
};

pub(crate) enum TrsiPropagatorWorkerEvent {
    Default(Hash),
    UpdateTransactionsReferencedByMilestone(HashSet<Hash>),
}

pub(crate) struct TrsiPropagatorWorker {
    pub(crate) tx: mpsc::UnboundedSender<TrsiPropagatorWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for TrsiPropagatorWorker {
    type Config = ();
    type Error = WorkerError;

    fn dependencies() -> &'static [TypeId] {
        Box::leak(Box::from(vec![TypeId::of::<TipCandidateValidatorWorker>()]))
    }

    async fn start(node: &N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded();
        let tip_candidate_validator = node.worker::<TipCandidateValidatorWorker>().unwrap().tx.clone();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx);

            while let Some(event) = receiver.next().await {
                let mut children = match &event {
                    TrsiPropagatorWorkerEvent::Default(hash) => vec![*hash],
                    TrsiPropagatorWorkerEvent::UpdateTransactionsReferencedByMilestone(hashes) => {
                        hashes.clone().into_iter().collect()
                    }
                };
                while let Some(hash) = children.pop() {
                    // get best otrsi and ytrsi from parents
                    let tx = tangle().get(&hash).unwrap();
                    let trunk_otsri = tangle().otrsi(tx.trunk());
                    let branch_otsri = tangle().otrsi(tx.branch());
                    let trunk_ytrsi = tangle().ytrsi(tx.trunk());
                    let branch_ytrsi = tangle().ytrsi(tx.branch());

                    if trunk_otsri.is_none()
                        || branch_otsri.is_none()
                        || trunk_ytrsi.is_none()
                        || branch_ytrsi.is_none()
                    {
                        continue;
                    }

                    // check if already confirmed by update_transactions_referenced_by_milestone()
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

                    // propagate otrsi and ytrsi to children
                    for child in tangle().get_children(&hash) {
                        children.push(child);
                    }

                    if let Err(e) =
                        tip_candidate_validator.unbounded_send(TipCandidateValidatorWorkerEvent::TrsiPropagated(hash))
                    {
                        warn!("Failed to send hash to tip candidate validator: {:?}.", e);
                    }
                }
                if let TrsiPropagatorWorkerEvent::UpdateTransactionsReferencedByMilestone(_) = event {
                    tangle().update_tip_pool();
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
