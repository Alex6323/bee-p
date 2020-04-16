use crate::{
    message::{
        Message,
        TransactionBroadcast,
    },
    protocol::Protocol,
};

use bee_network::{
    Command::SendMessage,
    EndpointId,
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

pub(crate) struct BroadcasterWorkerEvent {
    pub(crate) from: Option<EndpointId>,
    pub(crate) transaction_broadcast: TransactionBroadcast,
}

pub(crate) struct BroadcasterWorker {
    network: Network,
}

impl BroadcasterWorker {
    pub(crate) fn new(network: Network) -> Self {
        Self { network }
    }

    async fn broadcast(&mut self, from: Option<EndpointId>, bytes: Vec<u8>) {
        for entry in Protocol::get().contexts.iter() {
            if match from {
                Some(from) => from != *entry.key(),
                None => true,
            } {
                match self
                    .network
                    .send(SendMessage {
                        epid: *entry.key(),
                        bytes: bytes.clone(),
                        responder: None,
                    })
                    .await
                {
                    Ok(_) => {
                        // TODO metrics
                    }
                    Err(e) => {
                        warn!(
                            "[BroadcasterWorker ] Broadcasting transaction to {:?} failed: {:?}.",
                            *entry.key(),
                            e
                        );
                    }
                };
            }
        }
    }

    pub(crate) async fn run(
        mut self,
        receiver: mpsc::Receiver<BroadcasterWorkerEvent>,
        shutdown: oneshot::Receiver<()>,
    ) {
        info!("[BroadcasterWorker ] Running.");

        let mut receiver_fused = receiver.fuse();
        let mut shutdown_fused = shutdown.fuse();

        loop {
            select! {
                transaction = receiver_fused.next() => {
                    if let Some(BroadcasterWorkerEvent{from, transaction_broadcast}) = transaction {
                        self.broadcast(from, transaction_broadcast.into_full_bytes()).await;
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
