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

pub(crate) mod constants;
pub(crate) mod pruning;
pub(crate) mod worker;

pub mod config;
pub mod event;
pub mod global;
pub mod local;

use bee_common::{shutdown::Shutdown, shutdown_stream::ShutdownStream};
use bee_common_ext::event::Bus;
use bee_protocol::{event::LatestSolidMilestoneChanged, MilestoneIndex};

use async_std::task::spawn;
use futures::channel::{mpsc, oneshot};
use log::warn;

use std::{path::Path, sync::Arc};

pub enum Error {
    Global(global::Error),
    Local(local::Error),
    Download(local::DownloadError),
}

pub fn init(config: &config::SnapshotConfig, bus: Arc<Bus<'static>>, shutdown: &mut Shutdown) -> Result<(), Error> {
    match config.load_type() {
        config::LoadType::Global => {
            global::GlobalSnapshot::from_file(config.global().path(), MilestoneIndex(*config.global().index()))
                .map_err(Error::Global)?;
        }
        config::LoadType::Local => {
            if !Path::new(config.local().path()).exists() {
                local::download_local_snapshot(config.local()).map_err(Error::Download)?;
                // 		err = LoadSnapshotFromFile(path)
            }
        }
    }

    let (snapshot_worker_tx, snapshot_worker_rx) = mpsc::unbounded();
    let (snapshot_worker_shutdown_tx, snapshot_worker_shutdown_rx) = oneshot::channel();

    shutdown.add_worker_shutdown(
        snapshot_worker_shutdown_tx,
        spawn(
            worker::SnapshotWorker::new(
                config.clone(),
                ShutdownStream::new(snapshot_worker_shutdown_rx, snapshot_worker_rx),
            )
            .run(),
        ),
    );

    bus.add_listener(move |latest_solid_milestone: &LatestSolidMilestoneChanged| {
        if let Err(e) = snapshot_worker_tx.unbounded_send(worker::SnapshotWorkerEvent(latest_solid_milestone.0.clone()))
        {
            warn!(
                "Failed to send milestone {} to snapshot worker: {:?}.",
                *latest_solid_milestone.0.index(),
                e
            )
        }
    });

    Ok(())
}
