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
    message::{uncompress_transaction_bytes, Transaction as TransactionMessage},
    protocol::Protocol,
    tangle::{tangle, TransactionMetadata},
    worker::{milestone_validator::MilestoneValidatorWorkerEvent, transaction::HashCache},
};

use bee_common::worker::Error as WorkerError;
use bee_crypto::ternary::{
    sponge::{CurlP81, Sponge},
    Hash,
};
use bee_network::EndpointId;
use bee_tangle::traversal;
use bee_ternary::{T1B1Buf, T5B1Buf, Trits, T5B1};
use bee_transaction::bundled::{BundledTransaction as Transaction, BundledTransactionField};

use bytemuck::cast_slice;
use futures::{channel::mpsc, stream::StreamExt, SinkExt};
use log::{debug, error, info};

use std::time::{SystemTime, UNIX_EPOCH};

/// Timeframe to allow past or future transactions, 10 minutes in milliseconds.
const ALLOWED_TIMESTAMP_WINDOW_MS: u64 = 10 * 60 * 1000;

type Receiver = crate::worker::Receiver<mpsc::Receiver<TransactionWorkerEvent>>;

pub(crate) struct TransactionWorkerEvent {
    pub(crate) from: EndpointId,
    pub(crate) transaction: TransactionMessage,
}

pub(crate) struct TransactionWorker {
    milestone_validator_worker: mpsc::Sender<MilestoneValidatorWorkerEvent>,
    cache: HashCache,
    curl: CurlP81,
    receiver: Receiver,
}

impl TransactionWorker {
    pub(crate) fn new(
        milestone_validator_worker: mpsc::Sender<MilestoneValidatorWorkerEvent>,
        cache_size: usize,
        receiver: Receiver,
    ) -> Self {
        Self {
            milestone_validator_worker,
            cache: HashCache::new(cache_size),
            curl: CurlP81::new(),
            receiver,
        }
    }

    pub(crate) async fn run(mut self) -> Result<(), WorkerError> {
        info!("Running.");

        while let Some(TransactionWorkerEvent { from, transaction }) = self.receiver.next().await {
            self.process_transaction_brodcast(from, transaction).await;
        }

        info!("Stopped.");

        Ok(())
    }

    fn validate_timestamp(&self, transaction: &Transaction) -> (bool, bool) {
        // snapshotTimestamp := tangle.GetSnapshotInfo().Timestamp
        let timestamp = transaction.get_timestamp();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Clock may have gone backwards")
            .as_millis() as u64;

        let past = now - ALLOWED_TIMESTAMP_WINDOW_MS;
        let future = now + ALLOWED_TIMESTAMP_WINDOW_MS;

        // (is_timestamp_valid, should_broadcast)
        (
            timestamp >= Protocol::get().local_snapshot_timestamp && timestamp < future,
            timestamp >= past,
        )
    }

    async fn process_transaction_brodcast(&mut self, from: EndpointId, transaction_message: TransactionMessage) {
        debug!("Processing received transaction...");

        if !self.cache.insert(&transaction_message.bytes) {
            debug!("Transaction already received.");
            Protocol::get().metrics.known_transactions_inc();
            return;
        }

        let transaction_bytes = uncompress_transaction_bytes(&transaction_message.bytes);
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
                        Protocol::get().metrics.invalid_transactions_inc();
                        return;
                    }
                }
            }
            Err(e) => {
                debug!("Invalid transaction: {:?}.", e);
                Protocol::get().metrics.invalid_transactions_inc();
                return;
            }
        };

        if hash.weight() < Protocol::get().config.mwm {
            debug!("Insufficient weight magnitude: {}.", hash.weight());
            Protocol::get().metrics.invalid_transactions_inc();
            return;
        }

        let requested = Protocol::get().requested.contains_key(&hash);

        let (is_timestamp_valid, should_broadcast) = self.validate_timestamp(&transaction);

        if !requested && !is_timestamp_valid {
            debug!("Stale transaction, invalid timestamp.");
            Protocol::get().metrics.stale_transactions_inc();
            return;
        }

        let mut metadata = TransactionMetadata::new();

        if transaction.is_tail() {
            metadata.flags.set_tail();
        }
        if requested {
            metadata.flags.set_requested();
        }

        // store transaction
        if let Some(transaction) = tangle().insert(transaction, hash, metadata) {
            Protocol::get().metrics.new_transactions_inc();

            if !tangle().is_synced() && Protocol::get().requested.is_empty() {
                Protocol::trigger_milestone_solidification().await;
            }

            match Protocol::get().requested.remove(&hash) {
                Some((hash, index)) => {
                    Protocol::trigger_transaction_solidification(hash, index).await;
                }
                None => {
                    if should_broadcast {
                        Protocol::broadcast_transaction_message(Some(from), transaction_message).await
                    }
                }
            };

            if transaction.address().eq(&Protocol::get().config.coordinator.public_key)
                || transaction.address().eq(&Protocol::get().config.null_address)
            {
                let tail = {
                    if transaction.is_tail() {
                        Some(hash)
                    } else {
                        let mut last = None;

                        traversal::visit_children_follow_trunk(
                            tangle(),
                            hash,
                            |tx, _| tx.bundle() == transaction.bundle(),
                            |tx_hash, tx, _| {
                                last.replace((*tx_hash, tx.clone()));
                            },
                        );

                        if let Some((h, t)) = last {
                            if t.is_tail() {
                                Some(h)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                };

                if let Some(tail) = tail {
                    if let Err(e) = self
                        .milestone_validator_worker
                        .send(MilestoneValidatorWorkerEvent(tail))
                        .await
                    {
                        error!("Sending tail to milestone validation failed: {:?}.", e);
                    }
                };
            }
        } else {
            Protocol::get().metrics.known_transactions_inc();
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::tangle;

    use crate::config::ProtocolConfig;

    use bee_common::shutdown::Shutdown;
    use bee_common_ext::event::Bus;
    use bee_network::{NetworkConfig, Url};

    use async_std::task::{block_on, spawn};
    use futures::{channel::oneshot, sink::SinkExt};

    use std::sync::Arc;

    #[test]
    fn test_tx_worker_with_compressed_buffer() {
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

        let (transaction_worker_sender, transaction_worker_receiver) = mpsc::channel(1000);
        let (shutdown_sender, shutdown_receiver) = oneshot::channel();
        let (milestone_validator_worker_sender, _milestone_validator_worker_receiver) = mpsc::channel(1000);

        let mut transaction_worker_sender_clone = transaction_worker_sender;

        spawn(async move {
            let tx: [u8; 1024] = [0; 1024];
            let message = TransactionMessage::new(&tx);
            let epid: EndpointId = Url::from_url_str("tcp://[::1]:16000").await.unwrap().into();
            let event = TransactionWorkerEvent {
                from: epid,
                transaction: message,
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
            TransactionWorker::new(
                milestone_validator_worker_sender,
                10000,
                Receiver::new(transaction_worker_receiver, shutdown_receiver),
            )
            .run(),
        )
        .unwrap();

        assert_eq!(tangle().len(), 1);
        assert_eq!(tangle().contains(&Hash::zeros()), true);
    }
}
