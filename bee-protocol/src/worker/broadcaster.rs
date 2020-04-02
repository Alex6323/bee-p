use crate::{
    message::{
        Message,
        TransactionBroadcast,
    },
    protocol::Protocol,
};

use bee_network::{
    Command::SendMessage,
    Network,
};

use futures::{
    channel::{
        mpsc,
        oneshot,
    },
    future::FutureExt,
    select,
    stream::StreamExt,
};
use log::{
    info,
    warn,
};

pub(crate) type BroadcasterWorkerEvent = TransactionBroadcast;

pub(crate) struct BroadcasterWorker {
    network: Network,
    receiver: mpsc::Receiver<BroadcasterWorkerEvent>,
    shutdown: oneshot::Receiver<()>,
}

impl BroadcasterWorker {
    pub(crate) fn new(
        network: Network,
        receiver: mpsc::Receiver<BroadcasterWorkerEvent>,
        shutdown: oneshot::Receiver<()>,
    ) -> Self {
        Self {
            network,
            receiver,
            shutdown,
        }
    }

    pub(crate) async fn run(mut self) {
        info!("[BroadcasterWorker ] Running.");

        let mut receiver_fused = self.receiver.fuse();
        let mut shutdown_fused = self.shutdown.fuse();

        loop {
            select! {
                transaction = receiver_fused.next() => {
                    if let Some(transaction) = transaction {
                        let bytes = transaction.into_full_bytes();

                        for epid in Protocol::get().contexts.read().await.keys() {
                            match self
                                .network
                                .send(SendMessage {
                                    epid: *epid,
                                    bytes: bytes.clone(),
                                    responder: None,
                                })
                                .await {
                                Ok(_) => {
                                    // TODO metrics
                                },
                                Err(e) => {
                                    warn!("[BroadcasterWorker({}) ] Sending message failed: {}.", epid, e);
                                }
                            };
                        }
                    }
                },
                _ = shutdown_fused => {
                    break;
                }
            }
        }

        info!("[BroadcasterWorker ] Stopped.");
    }
}
