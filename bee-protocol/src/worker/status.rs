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

use bee_common::worker::Error as WorkerError;

use async_std::{future::ready, prelude::*};
use futures::channel::oneshot::Receiver;
use log::info;

use std::time::Duration;

pub(crate) struct StatusWorker {
    interval_ms: u64,
}

impl StatusWorker {
    pub(crate) fn new(interval_s: u64) -> Self {
        Self {
            interval_ms: interval_s * 1000,
        }
    }

    fn status(&self) {
        let snapshot_milestone_index = *tangle().get_snapshot_milestone_index();
        let solid_milestone_index = *tangle().get_last_solid_milestone_index();
        let last_milestone_index = *tangle().get_last_milestone_index();

        // TODO Threshold
        // TODO use tangle synced method
        let mut status = if solid_milestone_index == last_milestone_index {
            format!("Synchronized at {} -", last_milestone_index)
        } else {
            let progress = ((solid_milestone_index - snapshot_milestone_index) as f32 * 100.0
                / (last_milestone_index - snapshot_milestone_index) as f32) as u8;
            format!(
                "Synchronizing {}..{}..{} ({}%)",
                snapshot_milestone_index, solid_milestone_index, last_milestone_index, progress
            )
        };

        status = format!(
            "{} Requested {}",
            status,
            Protocol::get().requested_transactions.len()
        );

        info!("{}.", status);
    }

    pub(crate) async fn run(self, mut shutdown: Receiver<()>) -> Result<(), WorkerError> {
        info!("Running.");

        loop {
            match ready(Ok(()))
                .delay(Duration::from_millis(self.interval_ms))
                .race(&mut shutdown)
                .await
            {
                Ok(_) => self.status(),
                Err(_) => break,
            }
        }

        info!("Stopped.");

        Ok(())
    }
}
