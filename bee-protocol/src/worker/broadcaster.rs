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
    packet::{tlv_into_bytes, Message},
    protocol::Protocol,
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{node::Node, worker::Worker};
use bee_network::{Command::SendMessage, Network, PeerId};

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::{info, warn};

pub(crate) struct BroadcasterWorkerEvent {
    pub(crate) source: Option<PeerId>,
    pub(crate) message: Message,
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

        let network = config;

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            while let Some(BroadcasterWorkerEvent { source, message }) = receiver.next().await {
                let bytes = tlv_into_bytes(message);

                for peer in Protocol::get().peer_manager.peers.iter() {
                    if match source {
                        Some(ref source) => source != peer.key(),
                        None => true,
                    } {
                        match network.unbounded_send(SendMessage {
                            message: bytes.clone(),
                            to: peer.key().clone(),
                        }) {
                            Ok(_) => {
                                (*peer.value()).metrics.messages_sent_inc();
                                Protocol::get().metrics.messages_sent_inc();
                            }
                            Err(e) => {
                                warn!("Broadcasting message to {:?} failed: {:?}.", *peer.key(), e);
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
