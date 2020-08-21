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
    message::MilestoneRequest, milestone::MilestoneIndex, protocol::Protocol, tangle::tangle, worker::SenderWorker,
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::wait_priority_queue::WaitIncoming;
use bee_network::EndpointId;

use async_std::stream::{interval, Interval};
use futures::{select, stream::Fuse, FutureExt, StreamExt};
use log::info;

use std::{
    cmp::Ordering,
    time::{Duration, Instant},
};

const RETRY_INTERVAL_SECS: u64 = 5;

type Receiver<'a> = ShutdownStream<WaitIncoming<'a, MilestoneRequesterWorkerEntry>>;

#[derive(Eq, PartialEq)]
pub(crate) struct MilestoneRequesterWorkerEntry(pub(crate) MilestoneIndex, pub(crate) Option<EndpointId>);

impl PartialOrd for MilestoneRequesterWorkerEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    }
}

impl Ord for MilestoneRequesterWorkerEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.cmp(&self.0)
    }
}

pub(crate) struct MilestoneRequesterWorker<'a> {
    counter: usize,
    receiver: Receiver<'a>,
    timeouts: Fuse<Interval>,
}

impl<'a> MilestoneRequesterWorker<'a> {
    pub(crate) fn new(receiver: Receiver<'a>) -> Self {
        Self {
            counter: 0,
            receiver,
            timeouts: interval(Duration::from_secs(RETRY_INTERVAL_SECS)).fuse(),
        }
    }

    async fn process_request(&mut self, index: MilestoneIndex, epid: Option<EndpointId>) {
        if Protocol::get().requested_milestones.contains_key(&index) {
            return;
        }

        self.process_request_unchecked(index, epid).await
    }

    async fn process_request_unchecked(&mut self, index: MilestoneIndex, epid: Option<EndpointId>) {
        if Protocol::get().peer_manager.handshaked_peers.is_empty() {
            return;
        }

        match epid {
            Some(epid) => {
                if index.0 != 0 {
                    Protocol::get().requested_milestones.insert(index, Instant::now());
                }
                SenderWorker::<MilestoneRequest>::send(&epid, MilestoneRequest::new(*index));
            }
            None => {
                let guard = Protocol::get().peer_manager.handshaked_peers_keys.read().await;

                for _ in 0..guard.len() {
                    let epid = &guard[self.counter % guard.len()];

                    self.counter += 1;

                    if let Some(peer) = Protocol::get().peer_manager.handshaked_peers.get(epid) {
                        if index > peer.snapshot_milestone_index() && index <= peer.last_solid_milestone_index() {
                            if index.0 != 0 {
                                Protocol::get().requested_milestones.insert(index, Instant::now());
                            }
                            SenderWorker::<MilestoneRequest>::send(&epid, MilestoneRequest::new(*index));
                            break;
                        }
                    }
                }
            }
        }
    }

    async fn retry_requests(&mut self) {
        let mut retry_counts = 0;
        for mut tx in Protocol::get().requested_milestones.iter_mut() {
            let (index, instant) = tx.pair_mut();
            let now = Instant::now();
            if (now - *instant).as_secs() > RETRY_INTERVAL_SECS {
                *instant = now;
                self.process_request_unchecked(*index, None).await;
                retry_counts += 1;
            }
        }
        info!("Retried {} milestones", retry_counts);
    }

    pub(crate) async fn run(mut self) -> Result<(), WorkerError> {
        info!("Running.");

        loop {
            select! {
                // FIXME: Receiver is already fused
                entry = self.receiver.next().fuse() => match entry {
                    Some(MilestoneRequesterWorkerEntry(index, epid)) => {
                        if !tangle().contains_milestone(index.into()) {
                            self.process_request(index, epid).await;
                        }
                    },
                    None => break,
                },
                _ = self.timeouts.next() => self.retry_requests().await,
            }
        }

        info!("Stopped.");

        Ok(())
    }
}
