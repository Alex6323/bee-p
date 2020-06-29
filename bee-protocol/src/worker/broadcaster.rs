// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use crate::{
    message::{tlv_into_bytes, Transaction as TransactionMessage},
    protocol::Protocol,
};

use bee_network::{Command::SendMessage, EndpointId, Network};

use futures::{
    channel::{mpsc, oneshot},
    future::FutureExt,
    select,
    stream::StreamExt,
};
use log::{info, warn};

pub(crate) struct BroadcasterWorkerEvent {
    pub(crate) source: Option<EndpointId>,
    pub(crate) transaction: TransactionMessage,
}

pub(crate) struct BroadcasterWorker {
    network: Network,
}

impl BroadcasterWorker {
    pub(crate) fn new(network: Network) -> Self {
        Self { network }
    }

    async fn broadcast(&mut self, source: Option<EndpointId>, transaction: TransactionMessage) {
        let bytes = tlv_into_bytes(transaction);

        for peer in Protocol::get().peer_manager.handshaked_peers.iter() {
            if match source {
                Some(source) => source != *peer.key(),
                None => true,
            } {
                match self
                    .network
                    .send(SendMessage {
                        epid: *peer.key(),
                        bytes: bytes.clone(),
                        responder: None,
                    })
                    .await
                {
                    Ok(_) => {
                        (*peer.value()).metrics.transaction_sent_inc();
                        Protocol::get().metrics.transaction_sent_inc();
                    }
                    Err(e) => {
                        warn!("Broadcasting transaction to {:?} failed: {:?}.", *peer.key(), e);
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
        info!("Running.");

        let mut receiver_fused = receiver.fuse();
        let mut shutdown_fused = shutdown.fuse();

        loop {
            select! {
                transaction = receiver_fused.next() => {
                    if let Some(BroadcasterWorkerEvent{source, transaction}) = transaction {
                        self.broadcast(source, transaction).await;
                    }
                },
                _ = shutdown_fused => {
                    break;
                }
            }
        }

        info!("Stopped.");
    }
}
