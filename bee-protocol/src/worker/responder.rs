use crate::{
    message::{
        MilestoneRequest,
        TransactionBroadcast,
        TransactionRequest,
    },
    worker::{
        sender_registry,
        SenderWorkerEvent,
    },
};

use bee_network::EndpointId;

use futures::{
    channel::mpsc::Receiver,
    sink::SinkExt,
    stream::StreamExt,
};
use log::{
    info,
    warn,
};

pub enum ResponderWorkerEvent {
    TransactionRequest {
        epid: EndpointId,
        message: TransactionRequest,
    },
    MilestoneRequest {
        epid: EndpointId,
        message: MilestoneRequest,
    },
}

pub struct ResponderWorker {
    receiver: Receiver<ResponderWorkerEvent>,
}

impl ResponderWorker {
    pub fn new(receiver: Receiver<ResponderWorkerEvent>) -> Self {
        Self { receiver: receiver }
    }

    pub async fn run(mut self) {
        info!("[ResponderWorker ] Running.");

        while let Some(event) = self.receiver.next().await {
            if let (epid, Some(transaction)) = match event {
                ResponderWorkerEvent::TransactionRequest { epid, .. } => {
                    // TODO
                    // if let Some(transaction) = tangle.get_transaction(message.hash) {
                    //     (epid, Some(TransactionBroadcast::new(transaction.to_trits::<T5B1>()))
                    // }
                    // (epid, None)

                    // TODO remove
                    (epid, Some(TransactionBroadcast::new(&[0; 500])))
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

                    // TODO remove
                    (epid, Some(TransactionBroadcast::new(&[0; 500])))

                    // TODO send complete ms bundle ?
                }
            } {
                if let Some(context) = sender_registry().contexts().read().await.get(&epid) {
                    if let Err(e) = context
                        .transaction_broadcast_sender
                        // TODO avoid clone
                        .clone()
                        .send(SenderWorkerEvent::Message(transaction))
                        .await
                    {
                        warn!("[ResponderWorker ] Sending message failed: {}.", e);
                    }
                };
            }
        }
    }
}
