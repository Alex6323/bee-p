use crate::message::{Message, MilestoneRequest, TransactionBroadcast, TransactionRequest};

use bee_network::Command::SendBytes;
use bee_network::{EndpointId, Network};

use futures::channel::mpsc::Receiver;
use futures::stream::StreamExt;

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
    // TODO Dedicated sender with backpressure
    network: Network,
    receiver: Receiver<ResponderWorkerEvent>,
}

impl ResponderWorker {
    pub fn new(network: Network, receiver: Receiver<ResponderWorkerEvent>) -> Self {
        Self {
            network: network,
            receiver: receiver,
        }
    }

    pub async fn run(mut self) {
        while let Some(event) = self.receiver.next().await {
            if let (epid, Some(transaction)) = match event {
                ResponderWorkerEvent::TransactionRequest { epid, message } => {
                    // TODO
                    // if let Some(transaction) = tangle.get_transaction(message.hash) {
                    //     (epid, Some(TransactionBroadcast::new(transaction.to_trits::<T5B1>()))
                    // }
                    // (epid, None)

                    // TODO remove
                    (epid, Some(TransactionBroadcast::new(&[0; 500])))
                }
                ResponderWorkerEvent::MilestoneRequest { epid, message } => {
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
                self.network
                    .send(SendBytes {
                        epid: epid,
                        bytes: transaction.into_full_bytes(),
                        responder: None,
                    })
                    .await;
            }
        }
    }
}
