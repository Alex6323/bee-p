use crate::{
    message::{
        MilestoneRequest,
        TransactionBroadcast,
    },
    worker::SenderWorker,
};

use bee_bundle::Transaction;
use bee_network::EndpointId;
use bee_tangle::tangle;
use bee_ternary::{
    T1B1Buf,
    T5B1Buf,
    TritBuf,
};

use bytemuck::cast_slice;
use futures::{
    channel::{
        mpsc,
        oneshot,
    },
    future::FutureExt,
    select,
    stream::StreamExt,
};
use log::info;

pub(crate) struct MilestoneResponderWorkerEvent {
    pub(crate) epid: EndpointId,
    pub(crate) request: MilestoneRequest,
}

pub(crate) struct MilestoneResponderWorker {}

impl MilestoneResponderWorker {
    pub(crate) fn new() -> Self {
        Self {}
    }

    async fn process_request(&self, epid: EndpointId, request: MilestoneRequest) {
        let index = match request.index {
            0 => tangle().get_last_milestone_index(),
            _ => request.index.into(),
        };

        // TODO use get_milestone when fixed
        // TODO send complete ms bundle ?
        match tangle().get_milestone_hash(&index) {
            Some(hash) => match tangle().get_transaction(&hash).await {
                Some(transaction) => {
                    let mut trits = TritBuf::<T1B1Buf>::zeros(Transaction::trits_len());
                    transaction.into_trits_allocated(&mut trits);
                    SenderWorker::<TransactionBroadcast>::send(
                        &epid,
                        TransactionBroadcast::new(cast_slice(trits.encode::<T5B1Buf>().as_i8_slice())),
                    )
                    .await;
                }
                None => return,
            },
            None => return,
        }
    }

    pub(crate) async fn run(
        self,
        receiver: mpsc::Receiver<MilestoneResponderWorkerEvent>,
        shutdown: oneshot::Receiver<()>,
    ) {
        info!("[MilestoneResponderWorker ] Running.");

        let mut receiver_fused = receiver.fuse();
        let mut shutdown_fused = shutdown.fuse();

        loop {
            select! {
                event = receiver_fused.next() => {
                    if let Some(MilestoneResponderWorkerEvent { epid, request }) = event {
                        self.process_request(epid, request).await;
                    }
                },
                _ = shutdown_fused => {
                    break;
                }
            }
        }

        info!("[MilestoneResponderWorker ] Stopped.");
    }
}
