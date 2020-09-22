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

use crate::{protocol::Protocol, tangle::tangle};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{node::Node, worker::Worker};

use async_std::stream::{interval, Interval};
use async_trait::async_trait;
use futures::{stream::Fuse, StreamExt};
use log::info;

use std::time::Duration;

pub(crate) struct StatusWorker;

#[async_trait]
impl<N: Node + 'static> Worker<N> for StatusWorker {
    type Config = ();
    type Error = WorkerError;
    type Event = ();
    type Receiver = ShutdownStream<Fuse<Interval>>;

    async fn start(mut self, mut receiver: Self::Receiver, config: Self::Config) -> Result<(), Self::Error> {
        info!("Running.");

        while receiver.next().await.is_some() {
            self.status();
        }

        info!("Stopped.");

        Ok(())
    }
}

impl StatusWorker {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn interval(seconds: u64) -> Interval {
        interval(Duration::from_secs(seconds))
    }

    fn status(&self) {
        let snapshot_index = *tangle().get_snapshot_index();
        let latest_solid_milestone_index = *tangle().get_latest_solid_milestone_index();
        let latest_milestone_index = *tangle().get_latest_milestone_index();

        // TODO Threshold
        // TODO use tangle synced method
        if latest_solid_milestone_index == latest_milestone_index {
            info!("Synchronized at {}.", latest_milestone_index);
        } else {
            let progress = ((latest_solid_milestone_index - snapshot_index) as f32 * 100.0
                / (latest_milestone_index - snapshot_index) as f32) as u8;
            info!(
                "Synchronizing {}..{}..{} ({}%) - Requested {}.",
                snapshot_index,
                latest_solid_milestone_index,
                latest_milestone_index,
                progress,
                Protocol::get().requested_transactions.len()
            );
        };
    }
}
