use crate::{
    message::{
        MilestoneRequest,
        TransactionBroadcast,
    },
    worker::SenderWorker,
};

use bee_network::EndpointId;

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
    pub(crate) message: MilestoneRequest,
}

pub(crate) struct MilestoneResponderWorker {}

impl MilestoneResponderWorker {
    pub(crate) fn new() -> Self {
        Self {}
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
                    if let Some(MilestoneResponderWorkerEvent { epid, .. }) = event {
                        // TODO
                        // let index = if message.index == 0 {
                        //     tangle.get_latest_milestone_index()
                        // } else {
                        //     message.index
                        // }
                        // if let Some(transaction) = tangle.get_milestone(index) {
                        //     (epid, Some(TransactionBroadcast::new(transaction.to_trits::<T5B1>()))
                        // }
                        // (epid, None)

                        SenderWorker::<TransactionBroadcast>::send(&epid, TransactionBroadcast::new(&[0; 500])).await;
                        // TODO send complete ms bundle ?
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
