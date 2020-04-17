use crate::{
    milestone::{
        Milestone,
        MilestoneBuilder,
        MilestoneBuilderError,
    },
    protocol::Protocol,
};

use bee_bundle::Hash;
use bee_crypto::{
    Kerl,
    Sponge,
};
use bee_signing::{
    PublicKey,
    RecoverableSignature,
};
use bee_tangle::tangle;

use std::marker::PhantomData;

use futures::{
    channel::{
        mpsc,
        oneshot,
    },
    future::FutureExt,
    select,
    stream::StreamExt,
};
use log::{
    info,
    warn,
};

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
            .get_transaction(&tail_hash)
            .ok_or(MilestoneValidatorWorkerError::UnknownTail)?;

        if !transaction.is_tail() {
            Err(MilestoneValidatorWorkerError::NotATail)?;
        }

        builder.push((*transaction).clone());

        for _ in 0..Protocol::get().conf.coordinator.security_level {
            transaction = tangle()
                .get_transaction(transaction.trunk())
                .ok_or(MilestoneValidatorWorkerError::IncompleteBundle)?;

            builder.push((*transaction).clone());
        }

        Ok(builder
            .depth(Protocol::get().conf.coordinator.depth)
            .validate()
            .map_err(|e| MilestoneValidatorWorkerError::InvalidMilestone(e))?
            .build())
    }

    // TODO PriorityQueue ?
    pub(crate) async fn run(
        self,
        receiver: mpsc::Receiver<MilestoneValidatorWorkerEvent>,
        shutdown: oneshot::Receiver<()>,
    ) {
        info!("[MilestoneValidatorWorker ] Running.");

        let mut receiver_fused = receiver.fuse();
        let mut shutdown_fused = shutdown.fuse();

        loop {
            select! {
                tail_hash = receiver_fused.next() => {
                    if let Some(tail_hash) = tail_hash {
                        // TODO split
                        match self.validate_milestone(tail_hash).await {
                            Ok(milestone) => {
                                tangle().add_milestone_hash(milestone.index.into(), milestone.hash);
                                // TODO deref ? Why not .into() ?
                                if milestone.index > *tangle().get_last_milestone_index() {
                                    info!("[MilestoneValidatorWorker ] New milestone #{}.", milestone.index);
                                    tangle().update_last_milestone_index(milestone.index.into());
                                }
                                if milestone.index == *tangle().get_last_solid_milestone_index() + 1 {
                                    Protocol::trigger_milestone_solidification().await;
                                }
                            },
                            Err(e) => {
                                warn!("[MilestoneValidatorWorker ] Invalid milestone bundle: {:?}.", e);
                            }
                        }
                    }
                },
                _ = shutdown_fused => {
                    break;
                }
            }
        }

        info!("[MilestoneValidatorWorker ] Stopped.");
    }
}

#[cfg(test)]
mod tests {}
