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
    message::TransactionRequest,
    milestone::MilestoneIndex,
    protocol::{Protocol, Sender},
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_crypto::ternary::Hash;
use bee_ternary::T5B1Buf;

use async_std::stream::{interval, Interval};
use bytemuck::cast_slice;
use futures::{channel::mpsc, select, stream::Fuse, StreamExt};
use log::{debug, info};

use std::time::{Duration, Instant};

const RETRY_INTERVAL_SECS: u64 = 5;

type Receiver = ShutdownStream<mpsc::UnboundedReceiver<TransactionRequesterWorkerEntry>>;

pub(crate) struct TransactionRequesterWorkerEntry(pub(crate) Hash, pub(crate) MilestoneIndex);

pub(crate) struct TransactionRequesterWorker {
    counter: usize,
    receiver: Receiver,
    timeouts: Fuse<Interval>,
}

impl TransactionRequesterWorker {
    pub(crate) fn new(receiver: Receiver) -> Self {
        Self {
            counter: 0,
            receiver,
            timeouts: interval(Duration::from_secs(RETRY_INTERVAL_SECS)).fuse(),
        }
    }

    async fn process_request(&mut self, hash: Hash, index: MilestoneIndex) {
        if Protocol::get().requested_transactions.contains_key(&hash) {
            return;
        }

        if self.process_request_unchecked(hash, index).await {
            Protocol::get()
                .requested_transactions
                .insert(hash, (index, Instant::now()));
        }
    }

    /// Return `true` if the transaction was requested.
    async fn process_request_unchecked(&mut self, hash: Hash, index: MilestoneIndex) -> bool {
        if Protocol::get().peer_manager.handshaked_peers.is_empty() {
            return false;
        }

        let guard = Protocol::get().peer_manager.handshaked_peers_keys.read().await;

        for _ in 0..guard.len() {
            let epid = &guard[self.counter % guard.len()];

            self.counter += 1;

            if let Some(peer) = Protocol::get().peer_manager.handshaked_peers.get(epid) {
                if peer.has_data(index) {
                    Sender::<TransactionRequest>::send(
                        epid,
                        TransactionRequest::new(cast_slice(hash.as_trits().encode::<T5B1Buf>().as_i8_slice())),
                    )
                    .await;
                    return true;
                }
            }
        }

        for _ in 0..guard.len() {
            let epid = &guard[self.counter % guard.len()];

            self.counter += 1;

            if let Some(peer) = Protocol::get().peer_manager.handshaked_peers.get(epid) {
                if peer.maybe_has_data(index) {
                    let hash = hash.as_trits().encode::<T5B1Buf>();
                    Sender::<TransactionRequest>::send(
                        epid,
                        TransactionRequest::new(cast_slice(hash.as_i8_slice())),
                    );
                    return true;
                }
            }
        }

        false
    }

    async fn retry_requests(&mut self) {
        let mut retry_counts = 0;

        for mut transaction in Protocol::get().requested_transactions.iter_mut() {
            let (hash, (index, instant)) = transaction.pair_mut();
            let now = Instant::now();
            if (now - *instant).as_secs() > RETRY_INTERVAL_SECS && self.process_request_unchecked(*hash, *index).await {
                debug!(
                    "Retried request for transaction {} from milestone {}",
                    hash.iter_trytes().map(char::from).collect::<String>(),
                    **index
                );
                *instant = now;
                retry_counts += 1;
            }
        }

        if retry_counts > 0 {
            debug!("Retried {} transactions.", retry_counts);
        }
    }

    pub(crate) async fn run(mut self) -> Result<(), WorkerError> {
        info!("Running.");

        loop {
            select! {
                _ = self.timeouts.next() => self.retry_requests().await,
                entry = self.receiver.next() => match entry {
                    Some(TransactionRequesterWorkerEntry(hash, index)) => self.process_request(hash, index).await,
                    None => break,
                },
            }
        }

        info!("Stopped.");

        Ok(())
    }
}
