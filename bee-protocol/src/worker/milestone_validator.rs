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
    event::{LatestMilestoneChanged, LatestSolidMilestoneChanged},
    milestone::{Milestone, MilestoneBuilder, MilestoneBuilderError},
    protocol::Protocol,
    tangle::{helper::find_tail_of_bundle, tangle},
    worker::{MilestoneSolidifierWorker, MilestoneSolidifierWorkerEvent},
    MilestoneIndex,
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{node::Node, worker::Worker};
use bee_crypto::ternary::{
    sponge::{CurlP27, CurlP81, Kerl, Sponge, SpongeKind},
    Hash,
};
use bee_signing::ternary::{wots::WotsPublicKey, PublicKey, RecoverableSignature};
use bee_transaction::Vertex;

use async_trait::async_trait;
use futures::{channel::mpsc, stream::StreamExt};
use log::{debug, error, info};

use crate::{
    protocol::Sender,
    worker::{TrsiPropagatorWorker, TrsiPropagatorWorkerEvent},
};
use std::{any::TypeId, collections::HashSet};

#[derive(Debug)]
pub(crate) enum MilestoneValidatorWorkerError {
    UnknownTail,
    NotATail,
    IncompleteBundle,
    InvalidMilestone(MilestoneBuilderError),
}

pub(crate) struct MilestoneValidatorWorkerEvent(pub(crate) Hash, pub(crate) bool);

pub(crate) struct MilestoneValidatorWorker {
    pub(crate) tx: mpsc::UnboundedSender<MilestoneValidatorWorkerEvent>,
}

fn validate_milestone<N, M, P>(tail_hash: Hash) -> Result<Milestone, MilestoneValidatorWorkerError>
where
    N: Node,
    M: Sponge + Default + Send + Sync + 'static,
    P: PublicKey + Send + Sync + 'static,
    <P as PublicKey>::Signature: RecoverableSignature,
{
    // TODO also do an IncomingBundleBuilder check ?
    let mut builder = MilestoneBuilder::<Kerl, M, P>::new(tail_hash);
    let mut transaction = tangle()
        .get(&tail_hash)
        .ok_or(MilestoneValidatorWorkerError::UnknownTail)?;

    // TODO consider using the metadata instead as it might be more efficient
    if !transaction.is_tail() {
        return Err(MilestoneValidatorWorkerError::NotATail);
    }

    builder.push((*transaction).clone());

    // TODO use walker
    for _ in 0..Protocol::get().config.coordinator.security_level {
        transaction = tangle()
            .get((*transaction).trunk())
            .ok_or(MilestoneValidatorWorkerError::IncompleteBundle)?;

        builder.push((*transaction).clone());
    }

    Ok(builder
        .depth(Protocol::get().config.coordinator.depth)
        .validate()
        .map_err(MilestoneValidatorWorkerError::InvalidMilestone)?
        .build())
}

#[async_trait]
impl<N> Worker<N> for MilestoneValidatorWorker
where
    N: Node,
{
    type Config = SpongeKind;
    type Error = WorkerError;

    fn dependencies() -> &'static [TypeId] {
        Box::leak(Box::from(vec![TypeId::of::<MilestoneSolidifierWorker>()]))
    }

    async fn start(node: &N, config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded();
        let milestone_solidifier = node.worker::<MilestoneSolidifierWorker>().unwrap().tx.clone();
        let trsi_propagator = node.worker::<TrsiPropagatorWorker>().unwrap().tx.clone();

        let validate = match config {
            SpongeKind::Kerl => |hash| validate_milestone::<N, Kerl, WotsPublicKey<Kerl>>(hash),
            SpongeKind::CurlP27 => |hash| validate_milestone::<N, CurlP27, WotsPublicKey<CurlP27>>(hash),
            SpongeKind::CurlP81 => |hash| validate_milestone::<N, CurlP81, WotsPublicKey<CurlP81>>(hash),
        };

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx);

            while let Some(MilestoneValidatorWorkerEvent(hash, is_tail)) = receiver.next().await {
                let tail_hash = {
                    if is_tail {
                        Some(hash)
                    } else {
                        find_tail_of_bundle(tangle(), hash)
                    }
                };

                if let Some(tail_hash) = tail_hash {
                    if let Some(meta) = tangle().get_metadata(&tail_hash) {
                        if meta.flags().is_milestone() {
                            continue;
                        }
                        match validate(tail_hash) {
                            Ok(milestone) => {
                                tangle().add_milestone(milestone.index, milestone.hash);

                                // This is possibly not sufficient as there is no guarantee a milestone has been
                                // solidified before being validated, we then also need
                                // to check when a milestone gets solidified if it's
                                // already vadidated.
                                if meta.flags().is_solid() {
                                    Protocol::get()
                                        .bus
                                        .dispatch(LatestSolidMilestoneChanged(milestone.clone()));
                                }

                                if milestone.index > tangle().get_latest_milestone_index() {
                                    Protocol::get().bus.dispatch(LatestMilestoneChanged(milestone.clone()));
                                    let children =
                                        update_transactions_referenced_by_milestone(tail_hash, milestone.index);
                                    // Propagate updated OTRSI/YTRSI values to all direct/indirect approvers
                                    if let Err(e) = trsi_propagator.unbounded_send(
                                        TrsiPropagatorWorkerEvent::UpdateTransactionsReferencedByMilestone(children),
                                    ) {
                                        error!("Failed to send hash to TRSI propagator: {:?}.", e);
                                    }
                                }

                                if let Some(_) = Protocol::get().requested_milestones.remove(&milestone.index) {
                                    tangle()
                                        .update_metadata(&milestone.hash, |meta| meta.flags_mut().set_requested(true));

                                    milestone_solidifier
                                        .unbounded_send(MilestoneSolidifierWorkerEvent(milestone.index));
                                }
                            }
                            Err(e) => match e {
                                MilestoneValidatorWorkerError::IncompleteBundle => {}
                                _ => debug!("Invalid milestone bundle: {:?}.", e),
                            },
                        }
                    }
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}

// When a new milestone gets solid, OTRSI and YTRSI of all transactions that belong to the given milestone cone must be
// updated. This function returns all updated transactions.
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
