use crate::{
    milestone::{
        Milestone,
        MilestoneBuilder,
        MilestoneBuilderError,
    },
    protocol::Protocol,
};

use bee_bundle::{
    Hash,
    TransactionField,
};
use bee_tangle::tangle;

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
    IncompleteBundle,
    InvalidMilestone(MilestoneBuilderError),
}

pub(crate) type MilestoneValidatorWorkerEvent = Hash;

pub(crate) struct MilestoneValidatorWorker {}

impl MilestoneValidatorWorker {
    pub(crate) fn new() -> Self {
        Self {}
    }

    async fn validate_milestone(&self, tail_hash: Hash) -> Result<Milestone, MilestoneValidatorWorkerError> {
        let builder = MilestoneBuilder::new(tail_hash);

        let tail = match tangle().get_transaction(&tail_hash).await {
            Some(tail) => tail,
            None => Err(MilestoneValidatorWorkerError::UnknownTail)?,
        };

        // TODO clone :(
        // builder.push(tail.clone());

        let mut transaction = tail;
        // TODO bound ?
        for _ in 0..*tail.last_index().to_inner() {
            transaction = match tangle().get_transaction(transaction.trunk()).await {
                Some(transaction) => transaction,
                None => Err(MilestoneValidatorWorkerError::IncompleteBundle)?,
            };

            // TODO clone :(
            // builder.push(tail.clone());
        }

        Ok(builder
            .depth(Protocol::get().conf.coo_depth)
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
                        match self.validate_milestone(tail_hash).await {
                            Ok(milestone) => {
                                // TODO deref ? Why not .into() ?
                                if milestone.index > *tangle().get_last_milestone_index() {
                                    info!("[MilestoneValidatorWorker ] New milestone #{}.", milestone.index);
                                    tangle().update_last_milestone_index(milestone.index.into());
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
