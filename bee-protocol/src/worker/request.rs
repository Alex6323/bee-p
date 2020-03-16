use crate::message::{MilestoneRequest, TransactionRequest};

use netzwerk::PeerId;

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
    receiver: Receiver<RequestWorkerEvent>,
}

impl RequestWorker {
    pub(crate) fn new(receiver: Receiver<RequestWorkerEvent>) -> Self {
        Self { receiver: receiver }
    }

    pub(crate) async fn run(mut self) {
        while let Some(event) = self.receiver.next().await {
            match event {
                RequestWorkerEvent::TransactionRequest { peer_id, message } => {}
                RequestWorkerEvent::MilestoneRequest { peer_id, message } => {}
            }
        }
    }
}
