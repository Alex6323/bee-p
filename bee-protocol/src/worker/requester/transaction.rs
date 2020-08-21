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
    message::TransactionRequest, milestone::MilestoneIndex, protocol::Protocol, tangle::tangle, worker::SenderWorker,
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::wait_priority_queue::WaitIncoming;
use bee_crypto::ternary::Hash;
use bee_ternary::T5B1Buf;

use bytemuck::cast_slice;
use futures::StreamExt;
use log::info;

use std::cmp::Ordering;

type Receiver<'a> = ShutdownStream<WaitIncoming<'a, TransactionRequesterWorkerEntry>>;

#[derive(Eq, PartialEq)]
pub(crate) struct TransactionRequesterWorkerEntry(pub(crate) Hash, pub(crate) MilestoneIndex);

impl PartialOrd for TransactionRequesterWorkerEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.1.partial_cmp(&self.1)
    }
}

impl Ord for TransactionRequesterWorkerEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other.1.cmp(&self.1)
    }
}

pub(crate) struct TransactionRequesterWorker<'a> {
    counter: usize,
    receiver: Receiver<'a>,
}

impl<'a> TransactionRequesterWorker<'a> {
    pub(crate) fn new(receiver: Receiver<'a>) -> Self {
        Self { counter: 0, receiver }
    }

    async fn process_request(&mut self, hash: Hash, index: MilestoneIndex) {
        if Protocol::get().requested_transactions.contains_key(&hash) {
            return;
        }

        if Protocol::get().peer_manager.handshaked_peers.is_empty() {
            return;
        }

        let guard = Protocol::get().peer_manager.handshaked_peers_keys.read().await;

        for _ in 0..guard.len() {
            let epid = &guard[self.counter % guard.len()];

            self.counter += 1;

            if let Some(peer) = Protocol::get().peer_manager.handshaked_peers.get(epid) {
                if index > peer.snapshot_milestone_index() && index <= peer.last_solid_milestone_index() {
                    Protocol::get().requested_transactions.insert(hash, index);
                    SenderWorker::<TransactionRequest>::send(
                        epid,
                        TransactionRequest::new(cast_slice(hash.as_trits().encode::<T5B1Buf>().as_i8_slice())),
                    );
                    break;
                }
            }
        }
    }

    pub(crate) async fn run(mut self) -> Result<(), WorkerError> {
        info!("Running.");

        while let Some(TransactionRequesterWorkerEntry(hash, index)) = self.receiver.next().await {
            if !tangle().is_solid_entry_point(&hash) && !tangle().contains(&hash) {
                self.process_request(hash, index).await;
            }
        }

        info!("Stopped.");

        Ok(())
    }
}
