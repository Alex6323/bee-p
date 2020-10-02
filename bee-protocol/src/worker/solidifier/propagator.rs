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
    worker::{BundleValidatorWorker, BundleValidatorWorkerEvent},
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{node::Node, worker::Worker};
use bee_crypto::ternary::Hash;
use bee_transaction::Vertex;

use async_trait::async_trait;
use futures::{channel::mpsc, stream::StreamExt};
use log::{error, info, warn};

use crate::worker::milestone_cone_updater::{MilestoneConeUpdaterWorker, MilestoneConeUpdaterWorkerEvent};
use std::{
    any::TypeId,
    cmp::{max, min},
    collections::HashSet,
};

pub(crate) struct PropagatorWorkerEvent(pub(crate) Hash);

pub(crate) struct PropagatorWorker {
    pub(crate) tx: mpsc::UnboundedSender<PropagatorWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for PropagatorWorker {
    type Config = ();
    type Error = WorkerError;

    fn dependencies() -> &'static [TypeId] {
        Box::leak(Box::from(vec![
            TypeId::of::<BundleValidatorWorker>(),
            TypeId::of::<MilestoneConeUpdaterWorker>(),
        ]))
    }

    async fn start(node: &N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded();
        let bundle_validator = node.worker::<BundleValidatorWorker>().unwrap().tx.clone();
        let milestone_cone_updater = node.worker::<MilestoneConeUpdaterWorker>().unwrap().tx.clone();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx);

            while let Some(PropagatorWorkerEvent(hash)) = receiver.next().await {
                let mut children = vec![hash];

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

                    if !tangle().is_solid_transaction(&hash) {
                        let mut index = None;
                        tangle().update_metadata(&hash, |metadata| {
                            metadata.solidify();

                            // This is possibly not sufficient as there is no guarantee a milestone has been
                            // validated before being solidified, we then also need
                            // to check when a milestone gets validated if it's
                            // already solid.
                            if metadata.flags().is_milestone() {
                                index = Some(metadata.milestone_index());
                            }

                            if metadata.flags().is_tail() {
                                if let Err(e) = bundle_validator.unbounded_send(BundleValidatorWorkerEvent(hash)) {
                                    warn!("Failed to send hash to bundle validator: {:?}.", e);
                                }
                            }
                        });

                        Protocol::get().bus.dispatch(TransactionSolidified(hash));

                        if let Some(index) = index {
                            Protocol::get()
                                .bus
                                .dispatch(LatestSolidMilestoneChanged(Milestone { hash, index }));
                            if let Err(e) = milestone_cone_updater
                                .unbounded_send(MilestoneConeUpdaterWorkerEvent(Milestone { hash, index }))
                            {
                                error!("Sending tail to milestone validation failed: {:?}.", e);
                            }
                        }
                    }

                    // propagate to children
                    for child in tangle().get_children(&hash) {
                        children.push(child);
                    }
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
