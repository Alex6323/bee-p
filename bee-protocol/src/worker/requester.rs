use crate::message::{Message, MilestoneRequest, TransactionRequest};

use netzwerk::Command::SendBytes;
use netzwerk::Network;

use futures::channel::mpsc::Receiver;
use futures::stream::StreamExt;

pub(crate) enum RequesterWorkerEvent {
    TransactionRequest([u8; 49]),
    // TODO use MilestonIndex
    MilestoneRequest(u32),
}

pub(crate) struct RequesterWorker {
    // TODO network or dedicated sender ?
    network: Network,
    receiver: Receiver<RequesterWorkerEvent>,
}

impl RequesterWorker {
    pub(crate) fn new(network: Network, receiver: Receiver<RequesterWorkerEvent>) -> Self {
        Self {
            network: network,
            receiver: receiver,
        }
    }

    pub(crate) async fn run(mut self) {
        while let Some(event) = self.receiver.next().await {
            if let bytes = match event {
                RequesterWorkerEvent::TransactionRequest(hash) => TransactionRequest::new(hash).into_full_bytes(),
                RequesterWorkerEvent::MilestoneRequest(index) => MilestoneRequest::new(index).into_full_bytes(),
            } {
                // TODO we don't have any peer_id here
                // self.network
                //     .send(SendBytes {
                //         to_peer: peer_id,
                //         bytes: transaction.into_full_bytes(),
                //     })
                //     .await;
            }
        }
    }
}
