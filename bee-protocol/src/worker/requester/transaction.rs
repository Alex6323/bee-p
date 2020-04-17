use crate::{
    message::{
        Message,
        TransactionRequest,
    },
    milestone::MilestoneIndex,
    protocol::Protocol,
};

use bee_bundle::Hash;
use bee_tangle::tangle;
use bee_ternary::T5B1Buf;

use std::cmp::Ordering;

use bytemuck::cast_slice;
use futures::{
    channel::oneshot,
    future::FutureExt,
    select,
};
use log::info;

#[derive(Eq, PartialEq)]
pub(crate) struct TransactionRequesterWorkerEntry(pub(crate) Hash, pub(crate) MilestoneIndex);

// TODO check that this is the right order
impl PartialOrd for TransactionRequesterWorkerEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.1.partial_cmp(&other.1)
    }
}

impl Ord for TransactionRequesterWorkerEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.1.cmp(&other.1)
    }
}

pub(crate) struct TransactionRequesterWorker {}

impl TransactionRequesterWorker {
    pub(crate) fn new() -> Self {
        Self {}
    }

    fn process_request(&self, hash: Hash, _index: MilestoneIndex) {
        let _bytes =
            TransactionRequest::new(cast_slice(hash.as_trits().encode::<T5B1Buf>().as_i8_slice())).into_full_bytes();

        // TODO use sender worker
        // TODO check that neighbor may have the tx (by the index)
        // TODO convert hash to bytes
        // TODO we don't have any peer_id here
    }

    pub(crate) async fn run(self, shutdown: oneshot::Receiver<()>) {
        info!("[TransactionRequesterWorker ] Running.");

        let mut shutdown_fused = shutdown.fuse();

        loop {
            select! {
                // TODO impl fused stream
                entry = Protocol::get().transaction_requester_worker.0.pop().fuse() => {
                    if let TransactionRequesterWorkerEntry(hash, index) = entry {
                        if !tangle().is_solid_entry_point(&hash) && !tangle().contains_transaction(&hash) {
                            self.process_request(hash, index);
                        }
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
