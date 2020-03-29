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

// Transaction responder worker

pub(crate) enum TransactionResponderWorkerEvent {
    Request {
        epid: EndpointId,
        message: TransactionRequest,
    },
}

pub(crate) struct TransactionResponderWorker {
    receiver: Receiver<TransactionResponderWorkerEvent>,
}

impl TransactionResponderWorker {
    pub(crate) fn new(receiver: Receiver<TransactionResponderWorkerEvent>) -> Self {
        Self { receiver: receiver }
    }

    pub(crate) async fn run(mut self) {
        info!("[TransactionResponderWorker ] Running.");

        while let Some(event) = self.receiver.next().await {
            match event {
                TransactionResponderWorkerEvent::Request { epid, .. } => {
                    // TODO
                    // if let Some(transaction) = tangle.get_transaction(message.hash) {
                    //     (epid, Some(TransactionBroadcast::new(transaction.to_trits::<T5B1>()))
                    // }
                    // (epid, None)

                    SenderWorker::<TransactionBroadcast>::send(&epid, TransactionBroadcast::new(&[0; 500])).await;
                }
            }
        }
    }
}

// Milestone responder worker

pub(crate) enum MilestoneResponderWorkerEvent {
    Request {
        epid: EndpointId,
        message: MilestoneRequest,
    },
}

pub(crate) struct MilestoneResponderWorker {
    receiver: Receiver<MilestoneResponderWorkerEvent>,
}

impl MilestoneResponderWorker {
    pub(crate) fn new(receiver: Receiver<MilestoneResponderWorkerEvent>) -> Self {
        Self { receiver: receiver }
    }

    pub(crate) async fn run(mut self) {
        info!("[MilestoneResponderWorker ] Running.");

        while let Some(event) = self.receiver.next().await {
            match event {
                MilestoneResponderWorkerEvent::Request { epid, .. } => {
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
