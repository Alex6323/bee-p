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
    worker::{
        BroadcasterWorker, BroadcasterWorkerEvent, MilestoneValidatorWorker, MilestoneValidatorWorkerEvent,
        PropagatorWorker, PropagatorWorkerEvent, TransactionRequesterWorker,
    },
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{node::Node, worker::Worker};
use bee_crypto::ternary::Hash;
use bee_network::EndpointId;
use bee_ternary::{T1B1Buf, T5B1Buf, Trits, T5B1};
use bee_transaction::{
    bundled::{BundledTransaction as Transaction, TRANSACTION_TRIT_LEN},
    Vertex,
};

use async_trait::async_trait;
use bytemuck::cast_slice;
use futures::stream::StreamExt;
use log::{error, info, trace, warn};

use std::{
    any::TypeId,
    time::{SystemTime, UNIX_EPOCH},
};

pub(crate) struct ProcessorWorkerEvent {
    pub(crate) hash: Hash,
    pub(crate) from: EndpointId,
    pub(crate) transaction_message: TransactionMessage,
}

pub(crate) struct ProcessorWorker {
    pub(crate) tx: flume::Sender<ProcessorWorkerEvent>,
}

/// Timeframe to allow past or future transactions, 10 minutes in seconds.
const ALLOWED_TIMESTAMP_WINDOW_SECS: u64 = 10 * 60;

fn validate_timestamp(transaction: &Transaction) -> (bool, bool) {
    let timestamp = transaction.get_timestamp();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Clock may have gone backwards")
        .as_secs() as u64;
    let past = now - ALLOWED_TIMESTAMP_WINDOW_SECS;
    let future = now + ALLOWED_TIMESTAMP_WINDOW_SECS;

    // (is_timestamp_valid, should_broadcast)
    (
        timestamp >= Protocol::get().local_snapshot_timestamp && timestamp < future,
        timestamp >= past,
    )
}

#[async_trait]
impl<N: Node> Worker<N> for ProcessorWorker {
    type Config = ();
    type Error = WorkerError;

    fn dependencies() -> &'static [TypeId] {
        Box::leak(Box::from(vec![
            TypeId::of::<MilestoneValidatorWorker>(),
            TypeId::of::<PropagatorWorker>(),
            TypeId::of::<BroadcasterWorker>(),
            TypeId::of::<TransactionRequesterWorker>(),
        ]))
    }

    async fn start(node: &N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();
        let milestone_validator = node.worker::<MilestoneValidatorWorker>().unwrap().tx.clone();
        let propagator = node.worker::<PropagatorWorker>().unwrap().tx.clone();
        let broadcaster = node.worker::<BroadcasterWorker>().unwrap().tx.clone();
        let transaction_requester = node.worker::<TransactionRequesterWorker>().unwrap().tx.clone();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            while let Some(ProcessorWorkerEvent {
                hash,
                from,
                transaction_message,
            }) = receiver.next().await
            {
                trace!("Processing received transaction...");

                let transaction_bytes = uncompress_transaction_bytes(&transaction_message.bytes);
                let transaction =
                    match Trits::<T5B1>::try_from_raw(cast_slice(&transaction_bytes), TRANSACTION_TRIT_LEN) {
                        Ok(transaction_trits) => {
                            let transaction_buf = transaction_trits.to_buf::<T5B1Buf>().encode::<T1B1Buf>();
                            match Transaction::from_trits(&transaction_buf) {
                                Ok(transaction) => transaction,
                                Err(e) => {
                                    trace!("Invalid transaction: {:?}.", e);
                                    Protocol::get().metrics.invalid_transactions_inc();
                                    return;
                                }
                            }
                        }
                        Err(e) => {
                            trace!("Invalid transaction: {:?}.", e);
                            Protocol::get().metrics.invalid_transactions_inc();
                            return;
                        }
                    };

                let requested = Protocol::get().requested_transactions.contains_key(&hash);

                if !requested && hash.weight() < Protocol::get().config.mwm {
                    trace!("Insufficient weight magnitude: {}.", hash.weight());
                    Protocol::get().metrics.invalid_transactions_inc();
                    return;
                }

                let (is_timestamp_valid, should_broadcast) = validate_timestamp(&transaction);

                if !requested && !is_timestamp_valid {
                    trace!("Stale transaction, invalid timestamp.");
                    Protocol::get().metrics.stale_transactions_inc();
                    return;
                }

                let mut metadata = TransactionMetadata::arrived();

                metadata.flags_mut().set_tail(transaction.is_tail());
                metadata.flags_mut().set_requested(requested);

                // store transaction
                if let Some(transaction) = tangle().insert(transaction, hash, metadata) {
                    // TODO this was temporarily moved from the tangle.
                    // Reason is that since the tangle is not a worker, it can't have access to the propagator tx.
                    // When the tangle is made a worker, this should be put back on.

                    if let Err(e) = propagator.send(PropagatorWorkerEvent(hash)) {
                        error!("Failed to send hash to propagator: {:?}.", e);
                    }

                    Protocol::get().metrics.new_transactions_inc();

                    match Protocol::get().requested_transactions.remove(&hash) {
                        Some((_, (index, _))) => {
                            let trunk = transaction.trunk();
                            let branch = transaction.branch();

                            Protocol::request_transaction(&transaction_requester, *trunk, index);

                            if trunk != branch {
                                Protocol::request_transaction(&transaction_requester, *branch, index);
                            }
                        }
                        None => {
                            if should_broadcast {
                                if let Err(e) = broadcaster.send(BroadcasterWorkerEvent {
                                    source: Some(from),
                                    transaction: transaction_message,
                                }) {
                                    warn!("Broadcasting transaction failed: {}.", e);
                                }
                            }
                        }
                    };

                    if transaction.address().eq(&Protocol::get().config.coordinator.public_key) {
                        if let Err(e) =
                            milestone_validator.send(MilestoneValidatorWorkerEvent(hash, transaction.is_tail()))
                        {
                            error!("Sending tail to milestone validation failed: {:?}.", e);
                        }
                    }
                } else {
                    Protocol::get().metrics.known_transactions_inc();
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
