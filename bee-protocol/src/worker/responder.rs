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
    // TODO network or dedicated sender ?
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
                    // TODO Tangle lookup
                    (peer_id, Some(TransactionBroadcast::new(&[0; 500])))
                }
                ResponderWorkerEvent::MilestoneRequest { peer_id, message } => {
                    // TODO Tangle lookup
                    (peer_id, Some(TransactionBroadcast::new(&[0; 500])))
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
