use crate::{
    message::{
        MilestoneRequest,
        TransactionBroadcast,
        TransactionRequest,
    },
    worker::SenderWorker,
};

use bee_network::EndpointId;

use futures::{
    channel::mpsc::Receiver,
    stream::StreamExt,
};
use log::info;

pub(crate) enum ResponderWorkerEvent {
    TransactionRequest {
        epid: EndpointId,
        message: TransactionRequest,
    },
    MilestoneRequest {
        epid: EndpointId,
        message: MilestoneRequest,
    },
}

pub(crate) struct ResponderWorker {
    receiver: Receiver<ResponderWorkerEvent>,
}

impl ResponderWorker {
    pub(crate) fn new(receiver: Receiver<ResponderWorkerEvent>) -> Self {
        Self { receiver: receiver }
    }

    pub(crate) async fn run(mut self) {
        info!("[ResponderWorker ] Running.");

        while let Some(event) = self.receiver.next().await {
            match event {
                ResponderWorkerEvent::TransactionRequest { epid, .. } => {
                    // TODO
                    // if let Some(transaction) = tangle.get_transaction(message.hash) {
                    //     (epid, Some(TransactionBroadcast::new(transaction.to_trits::<T5B1>()))
                    // }
                    // (epid, None)

                    SenderWorker::<TransactionBroadcast>::send(&epid, TransactionBroadcast::new(&[0; 500])).await;
                }
                ResponderWorkerEvent::MilestoneRequest { epid, .. } => {
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
            }
        }
    }
}
