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
    tangle::MsTangle,
    worker::TangleWorker,
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{node::Node, worker::Worker};
use bee_network::EndpointId;

use async_trait::async_trait;
use futures::{select, StreamExt};
use log::{debug, info};
use tokio::time::interval;

use std::{
    any::TypeId,
    time::{Duration, Instant},
};

const RETRY_INTERVAL_SECS: u64 = 5;

pub(crate) struct MilestoneRequesterWorkerEvent(pub(crate) MilestoneIndex, pub(crate) Option<EndpointId>);

pub(crate) struct MilestoneRequesterWorker {
    pub(crate) tx: flume::Sender<MilestoneRequesterWorkerEvent>,
}

async fn process_request(index: MilestoneIndex, epid: Option<EndpointId>, counter: &mut usize) {
    if Protocol::get().requested_milestones.contains_key(&index) {
        return;
    }

    process_request_unchecked(index, epid, counter).await;

    if index.0 != 0 {
        Protocol::get().requested_milestones.insert(index, Instant::now());
    }
}

/// Return `true` if the milestone was requested
async fn process_request_unchecked(index: MilestoneIndex, epid: Option<EndpointId>, counter: &mut usize) -> bool {
    if Protocol::get().peer_manager.handshaked_peers.is_empty() {
        return false;
    }

    match epid {
        Some(epid) => {
            Sender::<MilestoneRequest>::send(&epid, MilestoneRequest::new(*index));
            true
        }
        None => {
            let guard = Protocol::get().peer_manager.handshaked_peers_keys.read().await;

            for _ in 0..guard.len() {
                let epid = &guard[*counter % guard.len()];

                *counter += 1;

                if let Some(peer) = Protocol::get().peer_manager.handshaked_peers.get(epid) {
                    if peer.maybe_has_data(index) {
                        Sender::<MilestoneRequest>::send(&epid, MilestoneRequest::new(*index));
                        return true;
                    }
                }
            }

            false
        }
    }
}

async fn retry_requests(counter: &mut usize) {
    let mut retry_counts: usize = 0;

    for mut milestone in Protocol::get().requested_milestones.iter_mut() {
        let (index, instant) = milestone.pair_mut();
        let now = Instant::now();
        if (now - *instant).as_secs() > RETRY_INTERVAL_SECS && process_request_unchecked(*index, None, counter).await {
            *instant = now;
            retry_counts += 1;
        };
    }

    if retry_counts > 0 {
        debug!("Retried {} milestones.", retry_counts);
    }
}

#[async_trait]
impl<N: Node> Worker<N> for MilestoneRequesterWorker {
    type Config = ();
    type Error = WorkerError;

    fn dependencies() -> &'static [TypeId] {
        Box::leak(Box::from(vec![TypeId::of::<TangleWorker>()]))
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();

        let tangle = node.resource::<MsTangle<N::Backend>>().clone();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            let mut counter: usize = 0;
            let mut timeouts = interval(Duration::from_secs(RETRY_INTERVAL_SECS)).fuse();

            loop {
                select! {
                    _ = timeouts.next() => retry_requests(&mut counter).await,
                    entry = receiver.next() => match entry {
                        Some(MilestoneRequesterWorkerEvent(index, epid)) => {
                            if !tangle.contains_milestone(index.into()) {
                                process_request(index, epid, &mut counter).await;
                            }
                        },
                        None => break,
                    },
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
