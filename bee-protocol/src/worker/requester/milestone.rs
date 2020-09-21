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
    message::MilestoneRequest,
    milestone::MilestoneIndex,
    protocol::{Protocol, Sender},
    tangle::tangle,
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{node::Node, worker::Worker};
use bee_network::EndpointId;

use async_std::stream::{interval, Interval};
use async_trait::async_trait;
use futures::{channel::mpsc, select, stream::Fuse, StreamExt};
use log::{debug, info};

use std::time::{Duration, Instant};

const RETRY_INTERVAL_SECS: u64 = 5;

pub(crate) struct MilestoneRequesterWorkerEvent(pub(crate) MilestoneIndex, pub(crate) Option<EndpointId>);

pub(crate) struct MilestoneRequesterWorker {
    counter: usize,
    timeouts: Fuse<Interval>,
}

#[async_trait]
impl<N: Node + 'static> Worker<N> for MilestoneRequesterWorker {
    type Event = MilestoneRequesterWorkerEvent;
    type Receiver = ShutdownStream<mpsc::UnboundedReceiver<MilestoneRequesterWorkerEvent>>;

    async fn run(self, receiver: Self::Receiver) -> Result<(), WorkerError> {
        async fn aux<N: Node + 'static>(
            mut worker: MilestoneRequesterWorker,
            mut receiver: <MilestoneRequesterWorker as Worker<N>>::Receiver,
        ) -> Result<(), WorkerError> {
            info!("Running.");

            loop {
                select! {
                    _ = worker.timeouts.next() => worker.retry_requests().await,
                    entry = receiver.next() => match entry {
                        Some(MilestoneRequesterWorkerEvent(index, epid)) => {
                            if !tangle().contains_milestone(index.into()) {
                                worker.process_request(index, epid).await;
                            }
                        },
                        None => break,
                    },
                }
            }

            info!("Stopped.");

            Ok(())
        }

        aux::<N>(self, receiver).await
    }
}

impl MilestoneRequesterWorker {
    pub(crate) fn new() -> Self {
        Self {
            counter: 0,
            timeouts: interval(Duration::from_secs(RETRY_INTERVAL_SECS)).fuse(),
        }
    }

    async fn process_request(&mut self, index: MilestoneIndex, epid: Option<EndpointId>) {
        if Protocol::get().requested_milestones.contains_key(&index) {
            return;
        }

        if self.process_request_unchecked(index, epid).await && index.0 != 0 {
            Protocol::get().requested_milestones.insert(index, Instant::now());
        }
    }

    /// Return `true` if the milestone was requested
    async fn process_request_unchecked(&mut self, index: MilestoneIndex, epid: Option<EndpointId>) -> bool {
        if Protocol::get().peer_manager.handshaked_peers.is_empty() {
            return false;
        }

        match epid {
            Some(epid) => {
                Sender::<MilestoneRequest>::send(&epid, MilestoneRequest::new(*index)).await;
                true
            }
            None => {
                let guard = Protocol::get().peer_manager.handshaked_peers_keys.read().await;

                for _ in 0..guard.len() {
                    let epid = &guard[self.counter % guard.len()];

                    self.counter += 1;

                    if let Some(peer) = Protocol::get().peer_manager.handshaked_peers.get(epid) {
                        if peer.maybe_has_data(index) {
                            Sender::<MilestoneRequest>::send(&epid, MilestoneRequest::new(*index)).await;
                            return true;
                        }
                    }
                }

                false
            }
        }
    }

    async fn retry_requests(&mut self) {
        let mut retry_counts = 0;

        for mut milestone in Protocol::get().requested_milestones.iter_mut() {
            let (index, instant) = milestone.pair_mut();
            let now = Instant::now();
            if (now - *instant).as_secs() > RETRY_INTERVAL_SECS && self.process_request_unchecked(*index, None).await {
                *instant = now;
                retry_counts += 1;
            };
        }

        if retry_counts > 0 {
            debug!("Retried {} milestones.", retry_counts);
        }
    }
}
