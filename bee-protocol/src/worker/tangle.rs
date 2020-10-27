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

use crate::{tangle::MsTangle, worker::storage::StorageWorker, MilestoneIndex};

use bee_common::shutdown_stream::ShutdownStream;
use bee_common_ext::{node::Node, worker::Worker};
use bee_snapshot::SnapshotHeader;

use async_trait::async_trait;
use log::{error, warn};
use tokio::time::interval;

use std::{
    any::TypeId,
    convert::Infallible,
    time::{Duration, Instant},
};

pub struct TangleWorker;

#[async_trait]
impl<N: Node> Worker<N> for TangleWorker {
    type Config = SnapshotHeader;
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<StorageWorker>()].leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let storage = node.storage();
        let tangle = MsTangle::<N::Backend>::new(storage);

        node.register_resource(tangle);

        let tangle = node.resource::<MsTangle<N::Backend>>();

        tangle.update_latest_solid_milestone_index(config.sep_index().into());
        tangle.update_latest_milestone_index(config.sep_index().into());
        tangle.update_snapshot_index(config.sep_index().into());
        tangle.update_pruning_index(config.sep_index().into());
        tangle.add_milestone(config.sep_index().into(), *config.sep_id());

        // for message_id in config.solid_entry_points() {
        //     // TODO no more indices ? What about TRSI ?
        //     tangle.add_solid_entry_point(*message_id, MilestoneIndex(0));
        // }

        node.spawn::<Self, _, _>(|shutdown| async move {
            use futures::StreamExt;
            use std::time::Duration;
            use tokio::time::interval;

            let mut receiver = ShutdownStream::new(shutdown, interval(Duration::from_secs(1)));

            while receiver.next().await.is_some() {
                // println!("Tangle len = {}", tangle.len());
            }
        });

        Ok(Self)
    }

    async fn stop(self, node: &mut N) -> Result<(), Self::Error> {
        let tangle = if let Some(tangle) = node.remove_resource::<MsTangle<N::Backend>>() {
            tangle
        } else {
            warn!(
                "The tangle was still in use by other users when the tangle worker stopped. \
                This is a bug, but not a critical one. From here, we'll revert to polling the \
                tangle until other users are finished with it."
            );

            let poll_start = Instant::now();
            let poll_freq = 20;
            let mut interval = interval(Duration::from_millis(poll_freq));
            loop {
                match node.remove_resource::<MsTangle<N::Backend>>() {
                    Some(tangle) => break tangle,
                    None => {
                        if Instant::now().duration_since(poll_start) > Duration::from_secs(5) {
                            error!(
                                "Tangle shutdown polling period elapsed. The tangle will be dropped \
                            without proper shutdown. This should be considered a bug."
                            );
                            return Ok(());
                        } else {
                            interval.tick().await;
                        }
                    }
                }
            }
        };

        Ok(tangle.shutdown().await)
    }
}
