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
    message::{tlv_into_bytes, TransactionBroadcast},
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
        for entry in Protocol::get().peer_manager.handshaked_peers.iter() {
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
                        warn!("Broadcasting transaction to {:?} failed: {:?}.", *entry.key(), e);
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
                    if let Some(BroadcasterWorkerEvent{from, transaction_broadcast}) = transaction {
                        self.broadcast(from, tlv_into_bytes(transaction_broadcast)).await;
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
