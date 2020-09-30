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

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{node::Node, worker::Worker};
use bee_network::{Command::SendMessage, EndpointId, Network};

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::{info, warn};

pub(crate) struct BroadcasterWorkerEvent {
    pub(crate) source: Option<EndpointId>,
    pub(crate) transaction: TransactionMessage,
}

pub(crate) struct BroadcasterWorker {
    pub(crate) tx: flume::Sender<BroadcasterWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for BroadcasterWorker {
    type Config = Network;
    type Error = WorkerError;

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            while let Some(BroadcasterWorkerEvent { source, transaction }) = receiver.next().await {
                let bytes = tlv_into_bytes(transaction);

                for peer in Protocol::get().peer_manager.handshaked_peers.iter() {
                    if match source {
                        Some(source) => source != *peer.key(),
                        None => true,
                    } {
                        match config.unbounded_send(SendMessage {
                            receiver_epid: *peer.key(),
                            message: bytes.clone(),
                        }) {
                            Ok(_) => {
                                (*peer.value()).metrics.transactions_sent_inc();
                                Protocol::get().metrics.transactions_sent_inc();
                            }
                            Err(e) => {
                                warn!("Broadcasting transaction to {:?} failed: {:?}.", *peer.key(), e);
                            }
                        };
                    }
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
