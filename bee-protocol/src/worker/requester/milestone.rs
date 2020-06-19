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

use bee_network::EndpointId;

use futures::{channel::oneshot, future::FutureExt, select};
use log::info;

use std::cmp::Ordering;

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

pub(crate) struct MilestoneRequesterWorker {
    counter: usize,
}

impl MilestoneRequesterWorker {
    pub(crate) fn new() -> Self {
        Self { counter: 0 }
    }

    async fn process_request(&mut self, index: MilestoneIndex, epid: Option<EndpointId>) {
        if Protocol::get().peer_manager.handshaked_peers.is_empty() {
            return;
        }

        let guard = Protocol::get().peer_manager.handshaked_peers_keys.read().await;

        match epid {
            Some(epid) => {
                SenderWorker::<MilestoneRequest>::send(&epid, MilestoneRequest::new(*index)).await;
            }
            None => {
                for _ in 0..guard.len() {
                    let epid = &guard[self.counter % guard.len()];

                    self.counter += 1;

                    if let Some(peer) = Protocol::get().peer_manager.handshaked_peers.get(epid) {
                        if index > peer.snapshot_milestone_index() && index <= peer.solid_milestone_index() {
                            SenderWorker::<MilestoneRequest>::send(&epid, MilestoneRequest::new(*index)).await;
                            break;
                        }
                    }
                }
            }
        }
    }

    pub(crate) async fn run(mut self, shutdown: oneshot::Receiver<()>) {
        info!("Running.");

        let mut shutdown_fused = shutdown.fuse();

        loop {
            select! {
                entry = Protocol::get().milestone_requester_worker.0.pop() => {
                    if let MilestoneRequesterWorkerEntry(index, epid) = entry {
                        if !tangle().contains_milestone(index.into()) {
                            self.process_request(index, epid).await;
                        }

                    }
                },
                _ = shutdown_fused => {
                    break;
                }
            }
        }

        info!("Stopped.");
    }
}
