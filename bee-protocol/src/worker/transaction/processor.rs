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
    worker::milestone_validator::MilestoneValidatorWorkerEvent,
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
use futures::{channel::mpsc, stream::StreamExt};
use log::{error, info, trace};

use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

pub(crate) struct ProcessorWorkerEvent {
    pub(crate) hash: Hash,
    pub(crate) from: EndpointId,
    pub(crate) transaction_message: TransactionMessage,
}

#[derive(Default)]
pub(crate) struct ProcessorWorker {}

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
    type Config = mpsc::UnboundedSender<MilestoneValidatorWorkerEvent>;
    type Error = WorkerError;
    type Event = ProcessorWorkerEvent;
    type Receiver = mpsc::UnboundedReceiver<Self::Event>;

    async fn start(receiver: Self::Receiver, node: Arc<N>, config: Self::Config) -> Result<Self, Self::Error> {
        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, receiver);

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
                    Protocol::get().metrics.new_transactions_inc();

                    match Protocol::get().requested_transactions.remove(&hash) {
                        Some((_, (index, _))) => {
                            let trunk = transaction.trunk();
                            let branch = transaction.branch();

                            Protocol::request_transaction(*trunk, index);

                            if trunk != branch {
                                Protocol::request_transaction(*branch, index);
                            }
                        }
                        None => {
                            if should_broadcast {
                                Protocol::broadcast_transaction_message(Some(from), transaction_message)
                            }
                        }
                    };

                    if transaction.address().eq(&Protocol::get().config.coordinator.public_key) {
                        if let Err(e) =
                            config.unbounded_send(MilestoneValidatorWorkerEvent(hash, transaction.is_tail()))
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

        Ok(Self::default())
    }
}
