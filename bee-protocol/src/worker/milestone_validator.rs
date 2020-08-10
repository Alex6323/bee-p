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
    event::{LastMilestone, LastSolidMilestone},
    milestone::{Milestone, MilestoneBuilder, MilestoneBuilderError},
    protocol::Protocol,
    tangle::tangle,
};

use bee_common::worker::Error as WorkerError;
use bee_crypto::ternary::{
    sponge::{Kerl, Sponge},
    Hash,
};
use bee_signing::ternary::{PublicKey, RecoverableSignature};
use bee_transaction::Vertex;

use log::{debug, info};

use futures::{channel::mpsc, stream::StreamExt};
use std::marker::PhantomData;

type Receiver = crate::worker::Receiver<mpsc::Receiver<MilestoneValidatorWorkerEvent>>;

#[derive(Debug)]
pub(crate) enum MilestoneValidatorWorkerError {
    UnknownTail,
    NotATail,
    IncompleteBundle,
    InvalidMilestone(MilestoneBuilderError),
}

pub(crate) struct MilestoneValidatorWorkerEvent(pub(crate) Hash);

pub(crate) struct MilestoneValidatorWorker<M, P> {
    receiver: Receiver,
    mss_sponge: PhantomData<M>,
    public_key: PhantomData<P>,
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
            mss_sponge: PhantomData,
            public_key: PhantomData,
        }
    }

    async fn validate_milestone(&self, tail_hash: Hash) -> Result<Milestone, MilestoneValidatorWorkerError> {
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

    async fn process(&self, tail_hash: Hash) {
        // TODO split
        match self.validate_milestone(tail_hash).await {
            Ok(milestone) => {
                // TODO check multiple triggers
                tangle().add_milestone(milestone.index, milestone.hash);

                // This is possibly not sufficient as there is no guarantee a milestone has been solidified
                // before being validated, we then also need to check when a milestone gets solidified if it's
                // already vadidated.
                if let Some(meta) = tangle().get_metadata(&milestone.hash) {
                    if meta.flags.is_solid() {
                        Protocol::get().bus.dispatch(LastSolidMilestone(milestone.clone()));
                    }
                }

                if milestone.index > tangle().get_last_milestone_index() {
                    Protocol::get().bus.dispatch(LastMilestone(milestone));
                }

                // TODO only trigger if index == last solid index ?
                // TODO trigger only if requester is empty ? And unsynced ?
                // Protocol::trigger_transaction_solidification(milestone.hash).await;
            }
            Err(e) => match e {
                MilestoneValidatorWorkerError::IncompleteBundle => {}
                _ => debug!("Invalid milestone bundle: {:?}.", e),
            },
        }
    }

    // TODO PriorityQueue ?
    pub(crate) async fn run(mut self) -> Result<(), WorkerError> {
        info!("Running.");

        while let Some(MilestoneValidatorWorkerEvent(tail_hash)) = self.receiver.next().await {
            self.process(tail_hash).await;
        }

        info!("Stopped.");

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
