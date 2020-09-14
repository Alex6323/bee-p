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
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_crypto::ternary::{
    sponge::{Kerl, Sponge},
    Hash,
};
use bee_signing::ternary::{PublicKey, RecoverableSignature};
use bee_transaction::Vertex;

use futures::{
    channel::mpsc,
    stream::{Fuse, StreamExt},
};
use log::{debug, info};

use std::marker::PhantomData;

type Receiver = ShutdownStream<Fuse<mpsc::UnboundedReceiver<MilestoneValidatorWorkerEvent>>>;

#[derive(Debug)]
pub(crate) enum MilestoneValidatorWorkerError {
    UnknownTail,
    NotATail,
    IncompleteBundle,
    InvalidMilestone(MilestoneBuilderError),
}

pub(crate) struct MilestoneValidatorWorkerEvent(pub(crate) Hash, pub(crate) bool);

pub(crate) struct MilestoneValidatorWorker<M, P> {
    receiver: Receiver,
    marker: PhantomData<(M, P)>,
}

impl<M, P> MilestoneValidatorWorker<M, P>
where
    M: Sponge + Default,
    P: PublicKey,
    <P as PublicKey>::Signature: RecoverableSignature,
{
    pub(crate) fn new(receiver: Receiver) -> Self {
        Self {
            receiver,
            marker: PhantomData,
        }
    }

    fn validate_milestone(&self, tail_hash: Hash) -> Result<Milestone, MilestoneValidatorWorkerError> {
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

    fn process(&self, hash: Hash, is_tail: bool) {
        let tail_hash = {
            if is_tail {
                Some(hash)
            } else {
                find_tail_of_bundle(tangle(), hash)
            }
        };

        if let Some(tail_hash) = tail_hash {
            if let Some(mut meta) = tangle().get_metadata(&tail_hash) {
                if meta.flags.is_milestone() {
                    return;
                }
                match self.validate_milestone(tail_hash) {
                    Ok(milestone) => {
                        tangle().add_milestone(milestone.index, milestone.hash);

                        // This is possibly not sufficient as there is no guarantee a milestone has been solidified
                        // before being validated, we then also need to check when a milestone gets solidified if it's
                        // already vadidated.
                        if meta.flags.is_solid() {
                            Protocol::get()
                                .bus
                                .dispatch(LatestSolidMilestoneChanged(milestone.clone()));
                        }

                        if milestone.index > tangle().get_latest_milestone_index() {
                            Protocol::get().bus.dispatch(LatestMilestoneChanged(milestone.clone()));
                        }

                        if let Some(_) = Protocol::get().requested_milestones.remove(&milestone.index) {
                            tangle().update_metadata(&milestone.hash, |meta| meta.flags.set_requested(true));

                            let tx = tangle().get(&milestone.hash).unwrap();
                            tangle().update_metadata(tx.trunk(), |meta| meta.flags.set_requested(true));
                            tangle().update_metadata(tx.branch(), |meta| meta.flags.set_requested(true));

                            Protocol::trigger_milestone_solidification(milestone.index);
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

    // TODO PriorityQueue ?
    pub(crate) async fn run(mut self) -> Result<(), WorkerError> {
        info!("Running.");

        while let Some(MilestoneValidatorWorkerEvent(hash, is_tail)) = self.receiver.next().await {
            self.process(hash, is_tail);
        }

        info!("Stopped.");

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
