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

mod hash_cache;
mod hasher;
mod processor;

pub(crate) use hash_cache::HashCache;
pub(crate) use hasher::{HasherWorker, HasherWorkerEvent};
pub(crate) use processor::{ProcessorWorker, ProcessorWorkerEvent};

// FIXME: fix this test
// mod tests {
//
//     use super::*;
//     use crate::tangle;
//
//     use crate::config::ProtocolConfig;
//
//     use bee_common::shutdown::Shutdown;
//     use bee_common_ext::event::Bus;
//     use bee_network::{NetworkConfig, Url};
//
//     use async_std::task::{block_on, spawn};
//     use futures::{channel::oneshot, sink::SinkExt};
//
//     use std::sync::Arc;
//
//     #[test]
//     fn test_tx_worker_with_compressed_buffer() {
//         let mut shutdown = Shutdown::new();
//         let bus = Arc::new(Bus::default());
//
//         // build network
//         let network_config = NetworkConfig::build().finish();
//         let (network, _) = bee_network::init(network_config, &mut shutdown);
//
//         // init tangle
//         tangle::init();
//
//         // init protocol
//         let protocol_config = ProtocolConfig::build().finish();
//         block_on(Protocol::init(protocol_config, network, 0, bus, &mut shutdown));
//
//         assert_eq!(tangle().len(), 0);
//
//         let (transaction_worker_sender, transaction_worker_receiver) = mpsc::channel(1000);
//         let (shutdown_sender, shutdown_receiver) = oneshot::channel();
//         let (milestone_validator_worker_sender, _milestone_validator_worker_receiver) = mpsc::channel(1000);
//
//         let mut transaction_worker_sender_clone = transaction_worker_sender;
//
//         spawn(async move {
//             let tx: [u8; 1024] = [0; 1024];
//             let message = TransactionMessage::new(&tx);
//             let epid: EndpointId = Url::from_url_str("tcp://[::1]:16000").await.unwrap().into();
//             let event = TransactionWorkerEvent {
//                 from: epid,
//                 transaction: message,
//             };
//             transaction_worker_sender_clone.send(event).await.unwrap();
//         });
//
//         spawn(async move {
//             use async_std::task;
//             use std::time::Duration;
//             task::sleep(Duration::from_secs(1)).await;
//             shutdown_sender.send(()).unwrap();
//         });
//
//         block_on(
//             TransactionWorker::new(
//                 milestone_validator_worker_sender,
//                 10000,
//                 ShutdownStream::new(shutdown_receiver, transaction_worker_receiver),
//             )
//             .run(),
//         )
//         .unwrap();
//
//         assert_eq!(tangle().len(), 1);
//         assert_eq!(tangle().contains(&Hash::zeros()), true);
//     }
// }
