// TODO Rename ? Request/Requester confusion

use crate::message::{Message, MilestoneRequest, TransactionBroadcast, TransactionRequest};

use netzwerk::Command::SendBytes;
use netzwerk::{Network, PeerId};

use futures::channel::mpsc::Receiver;
use futures::stream::StreamExt;

pub(crate) enum RequestWorkerEvent {
    TransactionRequest {
        peer_id: PeerId,
        message: TransactionRequest,
    },
    MilestoneRequest {
        peer_id: PeerId,
        message: MilestoneRequest,
    },
}

pub(crate) struct RequestWorker {
    // TODO network or dedicated sender ?
    network: Network,
    receiver: Receiver<RequestWorkerEvent>,
}

impl RequestWorker {
    pub(crate) fn new(network: Network, receiver: Receiver<RequestWorkerEvent>) -> Self {
        Self {
            network: network,
            receiver: receiver,
        }
    }

    pub(crate) async fn run(mut self) {
        while let Some(event) = self.receiver.next().await {
            if let (peer_id, Some(transaction)) = match event {
                RequestWorkerEvent::TransactionRequest { peer_id, message } => {
                    // TODO Tangle lookup
                    (peer_id, Some(TransactionBroadcast::new(&[0; 500])))
                }
                RequestWorkerEvent::MilestoneRequest { peer_id, message } => {
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
