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
    milestone::{Milestone, MilestoneBuilder, MilestoneBuilderError},
    protocol::Protocol,
    tangle::tangle,
};

use bee_crypto::ternary::{Hash, Kerl, Sponge};
use bee_signing::ternary::{PublicKey, RecoverableSignature};
use bee_transaction::TransactionVertex;

use std::marker::PhantomData;

use futures::{
    channel::{mpsc, oneshot},
    future::FutureExt,
    select,
    stream::StreamExt,
};
use log::{debug, info};

#[derive(Debug)]
pub(crate) enum MilestoneValidatorWorkerError {
    UnknownTail,
    NotATail,
    IncompleteBundle,
    InvalidMilestone(MilestoneBuilderError),
}

pub(crate) type MilestoneValidatorWorkerEvent = Hash;

pub(crate) struct MilestoneValidatorWorker<M, P> {
    mss_sponge: PhantomData<M>,
    public_key: PhantomData<P>,
}

impl<M, P> MilestoneValidatorWorker<M, P>
where
    M: Sponge + Default,
    P: PublicKey,
    <P as PublicKey>::Signature: RecoverableSignature,
{
    pub(crate) fn new() -> Self {
        Self {
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
                tangle().add_milestone(milestone.index.into(), milestone.hash);

                // TODO deref ? Why not .into() ?
                if milestone.index > tangle().get_last_milestone_index() {
                    info!("New milestone #{}.", *milestone.index);
                    tangle().update_last_milestone_index(milestone.index.into());
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
    pub(crate) async fn run(
        self,
        receiver: mpsc::Receiver<MilestoneValidatorWorkerEvent>,
        shutdown: oneshot::Receiver<()>,
    ) {
        info!("Running.");

        let mut receiver_fused = receiver.fuse();
        let mut shutdown_fused = shutdown.fuse();

        loop {
            select! {
                tail_hash = receiver_fused.next() => {
                    if let Some(tail_hash) = tail_hash {
                        self.process(tail_hash).await;
                    }
                },
                _ = shutdown_fused => {
                    break;
                }
            }
        }

        info!("Stopped.");
    }
}

#[cfg(test)]
mod tests {}
