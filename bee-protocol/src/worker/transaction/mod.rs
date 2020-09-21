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

use crate::node::BeeNode;

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

    use bee_common::{shutdown::Shutdown, shutdown_stream::ShutdownStream};
    use bee_common_ext::{event::Bus, worker::Worker};
    use bee_crypto::ternary::Hash;
    use bee_network::{EndpointId, NetworkConfig, Url};

    use async_std::task::{self, block_on, spawn};
    use futures::{
        channel::{mpsc, oneshot},
        join,
    };

    use std::{sync::Arc, time::Duration};

    #[test]
    fn test_tx_workers_with_compressed_buffer() {
        let mut shutdown = Shutdown::new();
        let bus = Arc::new(Bus::default());

        // build network
        let network_config = NetworkConfig::build().finish();
        let (network, _) = bee_network::init(network_config, &mut shutdown);

        // init tangle
        tangle::init();

        // init protocol
        let protocol_config = ProtocolConfig::build().finish();
        block_on(Protocol::init(protocol_config, network, 0, bus, &mut shutdown));

        assert_eq!(tangle().len(), 0);

        let (hasher_worker_sender, hasher_worker_receiver) = mpsc::unbounded();
        let (hasher_worker_shutdown_sender, hasher_worker_shutdown_receiver) = oneshot::channel();
        let (processor_worker_sender, processor_worker_receiver) = mpsc::unbounded();
        let (processor_worker_shutdown_sender, processor_worker_shutdown_receiver) = oneshot::channel();
        let (milestone_validator_worker_sender, _milestone_validator_worker_receiver) = mpsc::unbounded();

        let hasher_handle = HasherWorker::<BeeNode>::new(processor_worker_sender).run(
            <HasherWorker<BeeNode> as Worker<BeeNode>>::Receiver::new(
                10000,
                ShutdownStream::new(hasher_worker_shutdown_receiver, hasher_worker_receiver),
            ),
        );

        let processor = ProcessorWorker::new(milestone_validator_worker_sender);
        let processor_handle = Worker::<BeeNode>::run(
            processor,
            ShutdownStream::new(processor_worker_shutdown_receiver, processor_worker_receiver),
        );

        spawn(async move {
            let tx: [u8; 1024] = [0; 1024];
            let message = TransactionMessage::new(&tx);
            let epid: EndpointId = Url::from_url_str("tcp://[::1]:16000").await.unwrap().into();
            let event = HasherWorkerEvent {
                from: epid,
                transaction: message,
            };
            hasher_worker_sender.unbounded_send(event).unwrap();
            task::sleep(Duration::from_secs(5)).await;
            hasher_worker_shutdown_sender.send(()).unwrap();
            processor_worker_shutdown_sender.send(()).unwrap();
        });

        let (hasher_result, processor_result) = block_on(async { join!(hasher_handle, processor_handle) });

        hasher_result.unwrap();
        processor_result.unwrap();

        assert_eq!(tangle().len(), 1);
        assert_eq!(tangle().contains(&Hash::zeros()), true);
    }
}
