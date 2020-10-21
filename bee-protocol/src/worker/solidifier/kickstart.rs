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
    milestone::MilestoneIndex,
    protocol::Protocol,
    tangle::MsTangle,
    worker::{MilestoneRequesterWorker, RequestedMilestones, TangleWorker},
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{node::Node, worker::Worker};

use async_trait::async_trait;
use futures::{channel::oneshot, StreamExt};
use log::{error, info};
use tokio::time::interval;

use std::{any::TypeId, time::Duration};

#[derive(Default)]
pub(crate) struct KickstartWorker {}

#[async_trait]
impl<N: Node> Worker<N> for KickstartWorker {
    type Config = (oneshot::Sender<MilestoneIndex>, u32);
    type Error = WorkerError;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<MilestoneRequesterWorker>(), TypeId::of::<TangleWorker>()].leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let milestone_requester = node.worker::<MilestoneRequesterWorker>().unwrap().tx.clone();

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let requested_milestones = node.resource::<RequestedMilestones>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, interval(Duration::from_secs(1)));

            while receiver.next().await.is_some() {
                let next_ms = *tangle.get_latest_solid_milestone_index() + 1;
                let latest_ms = *tangle.get_latest_milestone_index();

                if !Protocol::get().peer_manager.handshaked_peers.is_empty() && next_ms + config.1 < latest_ms {
                    Protocol::request_milestone(
                        &tangle,
                        &milestone_requester,
                        &*requested_milestones,
                        MilestoneIndex(next_ms),
                        None,
                    );
                    if config.0.send(MilestoneIndex(next_ms)).is_err() {
                        error!("Could not set first non-solid milestone");
                    }

                    for index in next_ms..(next_ms + config.1) {
                        Protocol::request_milestone(
                            &tangle,
                            &milestone_requester,
                            &*requested_milestones,
                            MilestoneIndex(index),
                            None,
                        );
                    }
                    break;
                }
            }

            info!("Stopped.");
        });

        Ok(Self::default())
    }
}
