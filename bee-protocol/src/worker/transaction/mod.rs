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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::ProtocolConfig,
        message::Transaction as TransactionMessage,
        protocol::Protocol,
        tangle::{self, tangle},
    };

    use bee_common::shutdown_stream::ShutdownStream;
    use bee_common_ext::{event::Bus, shutdown_tokio::Shutdown};
    use bee_crypto::ternary::Hash;
    use bee_network::{EndpointId, NetworkConfig};

    use futures::{
        channel::{mpsc, oneshot},
        join,
    };
    use tokio::{spawn, time::delay_for};

    use std::{sync::Arc, time::Duration};

    #[tokio::test]
    async fn test_tx_workers_with_compressed_buffer() {
        let mut shutdown = Shutdown::new();
        let bus = Arc::new(Bus::default());

        // build network
        let network_config = NetworkConfig::builder().finish();
        let (network, _) = bee_network::init(network_config, &mut shutdown).await;

        // init tangle
        tangle::init();

        // init protocol
        let protocol_config = ProtocolConfig::build().finish();
        Protocol::init(protocol_config, network, 0, bus, &mut shutdown).await;

        assert_eq!(tangle().len(), 0);

        let (hasher_worker_sender, hasher_worker_receiver) = mpsc::unbounded();
        let (hasher_worker_shutdown_sender, hasher_worker_shutdown_receiver) = oneshot::channel();
        let (processor_worker_sender, processor_worker_receiver) = mpsc::unbounded();
        let (processor_worker_shutdown_sender, processor_worker_shutdown_receiver) = oneshot::channel();
        let (milestone_validator_worker_sender, _milestone_validator_worker_receiver) = mpsc::unbounded();

        let hasher_handle = HasherWorker::new(
            processor_worker_sender,
            10000,
            ShutdownStream::new(hasher_worker_shutdown_receiver, hasher_worker_receiver),
        )
        .run();

        let processor_handle = ProcessorWorker::new(
            milestone_validator_worker_sender,
            ShutdownStream::new(processor_worker_shutdown_receiver, processor_worker_receiver),
        )
        .run();

        spawn(async move {
            let tx: [u8; 1024] = [0; 1024];
            let message = TransactionMessage::new(&tx);
            let epid = EndpointId::new();
            let event = HasherWorkerEvent {
                from: epid,
                transaction: message,
            };
            hasher_worker_sender.unbounded_send(event).unwrap();
            delay_for(Duration::from_secs(5)).await;
            hasher_worker_shutdown_sender.send(()).unwrap();
            processor_worker_shutdown_sender.send(()).unwrap();
        });

        let (hasher_result, processor_result) = join!(hasher_handle, processor_handle);

        hasher_result.unwrap();
        processor_result.unwrap();

        assert_eq!(tangle().len(), 1);
        assert_eq!(tangle().contains(&Hash::zeros()), true);
    }
}
