use crate::message::{Message, MilestoneRequest, TransactionBroadcast, TransactionRequest};

use netzwerk::Command::SendBytes;
use netzwerk::{Network, PeerId};

use futures::channel::mpsc::Receiver;
use futures::stream::StreamExt;

pub(crate) enum ResponderWorkerEvent {
    TransactionRequest {
        peer_id: PeerId,
        message: TransactionRequest,
    },
    MilestoneRequest {
        peer_id: PeerId,
        message: MilestoneRequest,
    },
}

pub(crate) struct ResponderWorker {
    // TODO Dedicated sender with backpressure
    network: Network,
    receiver: Receiver<ResponderWorkerEvent>,
}

impl ResponderWorker {
    pub(crate) fn new(network: Network, receiver: Receiver<ResponderWorkerEvent>) -> Self {
        Self {
            network: network,
            receiver: receiver,
        }
    }

    pub(crate) async fn run(mut self) {
        while let Some(event) = self.receiver.next().await {
            if let (peer_id, Some(transaction)) = match event {
                ResponderWorkerEvent::TransactionRequest { peer_id, message } => {
                    // TODO
                    // if let Some(transaction) = tangle.get_transaction(message.hash) {
                    //     (peer_id, Some(TransactionBroadcast::new(transaction.to_trits::<T5B1>()))
                    // }
                    // (peer_id, None)

                    // TODO remove
                    (peer_id, Some(TransactionBroadcast::new(&[0; 500])))
                }
                ResponderWorkerEvent::MilestoneRequest { peer_id, message } => {
                    // TODO
                    // let index = if message.index == 0 {
                    //     tangle.get_latest_milestone_index()
                    // } else {
                    //     message.index
                    // }
                    // if let Some(transaction) = tangle.get_milestone(index) {
                    //     (peer_id, Some(TransactionBroadcast::new(transaction.to_trits::<T5B1>()))
                    // }
                    // (peer_id, None)

                    // TODO remove
                    (peer_id, Some(TransactionBroadcast::new(&[0; 500])))

                    // TODO send complete ms bundle ?
                }
            } {
                self.network
                    .send(SendBytes {
                        to_peer: peer_id,
                        bytes: transaction.into_full_bytes(),
                    })
                    .await;
            }
        }
    }
}
