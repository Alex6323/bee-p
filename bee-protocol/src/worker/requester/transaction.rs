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
use bee_common_ext::{node::Node, worker::Worker};
use bee_crypto::ternary::Hash;
use bee_ternary::T5B1Buf;

use async_trait::async_trait;
use bytemuck::cast_slice;
use dashmap::DashMap;
use futures::{select, StreamExt};
use log::{debug, info};
use tokio::time::interval;

use std::{
    ops::Deref,
    time::{Duration, Instant},
};

const RETRY_INTERVAL_SECS: u64 = 5;

#[derive(Default)]
pub(crate) struct RequestedTransactions(DashMap<Hash, (MilestoneIndex, Instant)>);

impl Deref for RequestedTransactions {
    type Target = DashMap<Hash, (MilestoneIndex, Instant)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) struct TransactionRequesterWorkerEvent(pub(crate) Hash, pub(crate) MilestoneIndex);

pub(crate) struct TransactionRequesterWorker {
    pub(crate) tx: flume::Sender<TransactionRequesterWorkerEvent>,
}

async fn process_request(
    requested_transactions: &RequestedTransactions,
    hash: Hash,
    index: MilestoneIndex,
    counter: &mut usize,
) {
    if requested_transactions.contains_key(&hash) {
        return;
    }

    if process_request_unchecked(hash, index, counter).await {
        requested_transactions.insert(hash, (index, Instant::now()));
    }
}

/// Return `true` if the transaction was requested.
async fn process_request_unchecked(hash: Hash, index: MilestoneIndex, counter: &mut usize) -> bool {
    if Protocol::get().peer_manager.handshaked_peers.is_empty() {
        return false;
    }

    let guard = Protocol::get().peer_manager.handshaked_peers_keys.read().await;

    for _ in 0..guard.len() {
        let epid = &guard[*counter % guard.len()];

        *counter += 1;

        if let Some(peer) = Protocol::get().peer_manager.handshaked_peers.get(epid) {
            if peer.has_data(index) {
                let hash = hash.as_trits().encode::<T5B1Buf>();
                Sender::<TransactionRequest>::send(epid, TransactionRequest::new(cast_slice(hash.as_i8_slice())));
                return true;
            }
        }
    }

    for _ in 0..guard.len() {
        let epid = &guard[*counter % guard.len()];

        *counter += 1;

        if let Some(peer) = Protocol::get().peer_manager.handshaked_peers.get(epid) {
            if peer.maybe_has_data(index) {
                let hash = hash.as_trits().encode::<T5B1Buf>();
                Sender::<TransactionRequest>::send(epid, TransactionRequest::new(cast_slice(hash.as_i8_slice())));
                return true;
            }
        }
    }

    false
}

async fn retry_requests(requested_transactions: &RequestedTransactions, counter: &mut usize) {
    let mut retry_counts: usize = 0;

    for mut transaction in requested_transactions.iter_mut() {
        let (hash, (index, instant)) = transaction.pair_mut();
        let now = Instant::now();
        if (now - *instant).as_secs() > RETRY_INTERVAL_SECS && process_request_unchecked(*hash, *index, counter).await {
            *instant = now;
            retry_counts += 1;
        }
    }

    if retry_counts > 0 {
        debug!("Retried {} transactions.", retry_counts);
    }
}

#[async_trait]
impl<N: Node> Worker<N> for TransactionRequesterWorker {
    type Config = ();
    type Error = WorkerError;

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();

        let requested_transactions: RequestedTransactions = Default::default();
        node.register_resource(requested_transactions);
        let requested_transactions = node.resource::<RequestedTransactions>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            let mut counter: usize = 0;
            let mut timeouts = interval(Duration::from_secs(RETRY_INTERVAL_SECS)).fuse();

            loop {
                select! {
                    _ = timeouts.next() => retry_requests(&*requested_transactions,&mut counter).await,
                    entry = receiver.next() => match entry {
                        Some(TransactionRequesterWorkerEvent(hash, index)) =>
                            process_request(&*requested_transactions,hash, index, &mut counter).await,
                        None => break,
                    },
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }

    // TODO stop + remove_resource
}
