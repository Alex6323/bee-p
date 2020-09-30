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
    tangle::{helper::find_tail_of_bundle, MsTangle},
    worker::{MilestoneSolidifierWorker, MilestoneSolidifierWorkerEvent},
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{node::Node, worker::Worker};
use bee_crypto::ternary::{
    sponge::{CurlP27, CurlP81, Kerl, Sponge, SpongeKind},
    Hash,
};
use bee_signing::ternary::{wots::WotsPublicKey, PublicKey, RecoverableSignature};
use bee_transaction::Vertex;
use bee_storage::storage::Backend;

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::{debug, info};

use std::any::TypeId;

#[derive(Debug)]
pub(crate) enum MilestoneValidatorWorkerError {
    UnknownTail,
    NotATail,
    IncompleteBundle,
    InvalidMilestone(MilestoneBuilderError),
}

pub(crate) struct MilestoneValidatorWorkerEvent(pub(crate) Hash, pub(crate) bool);

pub(crate) struct MilestoneValidatorWorker {
    pub(crate) tx: flume::Sender<MilestoneValidatorWorkerEvent>,
}

async fn validate_milestone<N, M, P, B: Backend>(tangle: &MsTangle<B>, tail_hash: Hash) -> Result<Milestone, MilestoneValidatorWorkerError>
where
    N: Node,
    M: Sponge + Default + Send + Sync + 'static,
    P: PublicKey + Send + Sync + 'static,
    <P as PublicKey>::Signature: RecoverableSignature,
{
    // TODO also do an IncomingBundleBuilder check ?
    let mut builder = MilestoneBuilder::<Kerl, M, P>::new(tail_hash);
    let mut transaction = tangle
        .get(&tail_hash)
        .await
        .ok_or(MilestoneValidatorWorkerError::UnknownTail)?;

    // TODO consider using the metadata instead as it might be more efficient
    if !transaction.is_tail() {
        return Err(MilestoneValidatorWorkerError::NotATail);
    }

    builder.push((*transaction).clone());

    // TODO use walker
    for _ in 0..Protocol::get().config.coordinator.security_level {
        transaction = tangle
            .get((*transaction).trunk())
            .await
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

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();
        let milestone_solidifier = node.worker::<MilestoneSolidifierWorker>().unwrap().tx.clone();

        let tangle = node.resource::<MsTangle<N::Backend>>().clone();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            while let Some(MilestoneValidatorWorkerEvent(hash, is_tail)) = receiver.next().await {
                let tail_hash = if is_tail {
                    Some(hash)
                } else {
                    find_tail_of_bundle(&tangle, hash)
                };

                if let Some(tail_hash) = tail_hash {
                    if let Some(meta) = tangle.get_metadata(&tail_hash) {
                        if meta.flags().is_milestone() {
                            continue;
                        }

                        match match config {
                            SpongeKind::Kerl => validate_milestone::<N, Kerl, WotsPublicKey<Kerl>, N::Backend>(&tangle, tail_hash).await,
                            SpongeKind::CurlP27 => validate_milestone::<N, CurlP27, WotsPublicKey<CurlP27>, N::Backend>(&tangle, tail_hash).await,
                            SpongeKind::CurlP81 => validate_milestone::<N, CurlP81, WotsPublicKey<CurlP81>, N::Backend>(&tangle, tail_hash).await,
                        } {
                            Ok(milestone) => {
                                tangle.add_milestone(milestone.index, milestone.hash);

                                // This is possibly not sufficient as there is no guarantee a milestone has been
                                // solidified before being validated, we then also need
                                // to check when a milestone gets solidified if it's
                                // already vadidated.
                                if meta.flags().is_solid() {
                                    Protocol::get()
                                        .bus
                                        .dispatch(LatestSolidMilestoneChanged(milestone.clone()));
                                }

                                if milestone.index > tangle.get_latest_milestone_index() {
                                    Protocol::get().bus.dispatch(LatestMilestoneChanged(milestone.clone()));
                                }

                                if let Some(_) = Protocol::get().requested_milestones.remove(&milestone.index) {
                                    tangle
                                        .update_metadata(&milestone.hash, |meta| meta.flags_mut().set_requested(true));

                                    milestone_solidifier.send(MilestoneSolidifierWorkerEvent(milestone.index));
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
