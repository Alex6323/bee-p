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
    message::{uncompress_transaction_bytes, TransactionBroadcast},
    protocol::Protocol,
    tangle::{flags::Flags, tangle},
    worker::transaction::TinyHashCache,
};

use bee_crypto::{CurlP81, Sponge};
use bee_network::EndpointId;
use bee_tangle::traversal;
use bee_ternary::{T1B1Buf, T5B1Buf, Trits, T5B1};
use bee_transaction::{BundledTransaction as Transaction, BundledTransactionField, Hash};

use bytemuck::cast_slice;
use futures::{
    channel::{mpsc, oneshot},
    future::FutureExt,
    select,
    stream::StreamExt,
    SinkExt,
};
use log::{debug, error, info};

pub(crate) struct TransactionWorkerEvent {
    pub(crate) from: EndpointId,
    pub(crate) transaction_broadcast: TransactionBroadcast,
}

pub(crate) struct TransactionWorker {
    milestone_validator_worker: mpsc::Sender<Hash>,
    cache: TinyHashCache,
    curl: CurlP81,
}

impl TransactionWorker {
    pub(crate) fn new(milestone_validator_worker: mpsc::Sender<Hash>, cache_size: usize) -> Self {
        Self {
            milestone_validator_worker,
            cache: TinyHashCache::new(cache_size),
            curl: CurlP81::new(),
        }
    }

    pub(crate) async fn run(
        mut self,
        receiver: mpsc::Receiver<TransactionWorkerEvent>,
        shutdown: oneshot::Receiver<()>,
    ) {
        info!("Running.");

        let mut receiver_fused = receiver.fuse();
        let mut shutdown_fused = shutdown.fuse();

        loop {
            select! {
                event = receiver_fused.next() => {
                    if let Some(TransactionWorkerEvent{from, transaction_broadcast}) = event {
                        self.process_transaction_brodcast(from, transaction_broadcast).await;
                    }
                },
                _ = shutdown_fused => break
            }
        }

        info!("Stopped.");
    }

    async fn process_transaction_brodcast(&mut self, from: EndpointId, transaction_broadcast: TransactionBroadcast) {
        debug!("Processing received transaction...");

        if !self.cache.insert(&transaction_broadcast.transaction) {
            debug!("Transaction already received.");
            Protocol::get().metrics.known_transactions_received_inc();
            return;
        }

        let transaction_bytes = uncompress_transaction_bytes(&transaction_broadcast.transaction);
        let (transaction, hash) = match Trits::<T5B1>::try_from_raw(cast_slice(&transaction_bytes), 8019) {
            Ok(transaction_trits) => {
                let transaction_buf = transaction_trits.to_buf::<T5B1Buf>().encode::<T1B1Buf>();
                match Transaction::from_trits(&transaction_buf) {
                    Ok(transaction) => (
                        transaction,
                        Hash::from_inner_unchecked(self.curl.digest(&transaction_buf).unwrap()),
                    ),
                    Err(e) => {
                        debug!("Invalid transaction: {:?}.", e);
                        Protocol::get().metrics.invalid_transactions_received_inc();
                        return;
                    }
                }
            }
            Err(e) => {
                debug!("Invalid transaction: {:?}.", e);
                Protocol::get().metrics.invalid_transactions_received_inc();
                return;
            }
        };

        if hash.weight() < Protocol::get().config.mwm {
            debug!("Insufficient weight magnitude: {}.", hash.weight());
            Protocol::get().metrics.invalid_transactions_received_inc();
            return;
        }

        // store transaction
        if let Some(transaction) = tangle().insert(transaction, hash, Flags::empty()) {
            Protocol::get().metrics.new_transactions_received_inc();

            if !tangle().is_synced() && Protocol::get().requested.is_empty() {
                Protocol::trigger_milestone_solidification().await;
            }

            match Protocol::get().requested.remove(&hash) {
                Some((hash, index)) => {
                    Protocol::trigger_transaction_solidification(hash, index).await;
                }
                None => Protocol::broadcast_transaction_message(Some(from), transaction_broadcast).await,
            };

            if transaction.address().eq(&Protocol::get().config.coordinator.public_key)
                || transaction.address().eq(&Protocol::get().config.null_address)
            {
                let tail = {
                    if transaction.is_tail() {
                        Some(hash)
                    } else {
                        let mut bundle = vec![];

                        traversal::visit_children_follow_trunk(
                            &tangle().inner,
                            hash,
                            |tx, _| tx.bundle() == transaction.bundle(),
                            |tx_hash, tx, _| {
                                bundle.push((*tx_hash, tx.clone()));
                            },
                        );

                        match bundle.last() {
                            Some((h, t)) => {
                                if t.is_tail() {
                                    Some(*h)
                                } else {
                                    None
                                }
                            }
                            None => None,
                        }
                    }
                };

                if let Some(tail) = tail {
                    if let Err(e) = self.milestone_validator_worker.send(tail).await {
                        error!("Sending tail to milestone validation failed: {:?}.", e);
                    }
                };
            }
        } else {
            Protocol::get().metrics.known_transactions_received_inc();
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::ProtocolConfig;

    use bee_network::{NetworkConfig, Url};

    use async_std::task::{block_on, spawn};
    use futures::sink::SinkExt;

    #[test]
    fn test_tx_worker_with_compressed_buffer() {
        crate::tangle::init();

        // build network
        let network_config = NetworkConfig::build().finish();
        let (network, _shutdown, _receiver) = bee_network::init(network_config);

        // init protocol
        let protocol_config = ProtocolConfig::build().finish();
        block_on(Protocol::init(protocol_config, network));

        assert_eq!(tangle().size(), 0);

        let (transaction_worker_sender, transaction_worker_receiver) = mpsc::channel(1000);
        let (shutdown_sender, shutdown_receiver) = oneshot::channel();
        let (milestone_validator_worker_sender, _milestone_validator_worker_receiver) = mpsc::channel(1000);

        let mut transaction_worker_sender_clone = transaction_worker_sender;

        spawn(async move {
            let tx: [u8; 1024] = [0; 1024];
            let message = TransactionBroadcast::new(&tx);
            let epid: EndpointId = Url::from_url_str("tcp://[::1]:16000").await.unwrap().into();
            let event = TransactionWorkerEvent {
                from: epid,
                transaction_broadcast: message,
            };
            transaction_worker_sender_clone.send(event).await.unwrap();
        });

        spawn(async move {
            use async_std::task;
            use std::time::Duration;
            task::sleep(Duration::from_secs(1)).await;
            shutdown_sender.send(()).unwrap();
        });

        block_on(
            TransactionWorker::new(milestone_validator_worker_sender, 10000)
                .run(transaction_worker_receiver, shutdown_receiver),
        );

        assert_eq!(tangle().inner.size(), 1);
        assert_eq!(tangle().contains(&Hash::zeros()), true);
    }
}
