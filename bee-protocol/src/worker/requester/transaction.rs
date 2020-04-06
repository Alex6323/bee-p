use crate::{
    milestone::MilestoneIndex,
    protocol::Protocol,
};

use bee_bundle::Hash;

use std::cmp::Ordering;

use futures::{
    channel::oneshot,
    future::FutureExt,
    select,
};
use log::info;

#[derive(Eq, PartialEq)]
pub(crate) struct TransactionRequesterWorkerEntry(MilestoneIndex, Hash);

// TODO check that this is the right order
impl PartialOrd for TransactionRequesterWorkerEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for TransactionRequesterWorkerEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

pub(crate) struct TransactionRequesterWorker {}

impl TransactionRequesterWorker {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) async fn run(self, shutdown: oneshot::Receiver<()>) {
        info!("[TransactionRequesterWorker ] Running.");

        let mut shutdown_fused = shutdown.fuse();

        loop {
            select! {
                // TODO impl fused stream
                entry = Protocol::get().transaction_requester_worker.pop().fuse() => {
                    if let TransactionRequesterWorkerEntry(_, _hash) = entry {
                        // TODO cheeck that neighbor may have the tx (by the index)
                        // TODO convert hash to bytes
                        // let _bytes = TransactionRequest::new(hash).into_full_bytes();
                        // TODO we don't have any peer_id here
                    }
                },
                _ = shutdown_fused => {
                    break;
                }
            }
        }

        info!("[TransactionRequesterWorker ] Stopped.");
    }
}
