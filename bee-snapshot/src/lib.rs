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

mod download;

pub(crate) mod constants;
pub(crate) mod kind;
pub(crate) mod pruning;
// pub(crate) mod worker;

pub mod config;
pub mod event;
pub mod header;
pub mod metadata;
pub mod snapshot;

pub(crate) use download::{download_local_snapshot, Error as DownloadError};

use metadata::SnapshotMetadata;
use snapshot::{Error as FileError, LocalSnapshot};

use bee_common_ext::{event::Bus, node::Node};
use bee_message::prelude::MessageId;
// use bee_protocol::{event::LatestSolidMilestoneChanged, MilestoneIndex};

use chrono::{offset::TimeZone, Utc};
use log::info;

use std::{path::Path, sync::Arc};

#[derive(Debug)]
pub enum Error {
    Local(FileError),
    Download(DownloadError),
}

// TODO change return type

pub async fn init<N: Node>(
    // tangle: &MsTangle<B>,
    config: &config::SnapshotConfig,
    node_builder: N::Builder,
) -> Result<(N::Builder, SnapshotMetadata), Error> {
    if !Path::new(config.path()).exists() {
        download_local_snapshot(config).await.map_err(Error::Download)?;
    }
    info!("Loading local snapshot file {}...", config.path());

    let LocalSnapshot { mut metadata } = LocalSnapshot::from_file(config.path()).map_err(Error::Local)?;

    info!(
        "Loaded local snapshot file from {} with {} solid entry points.",
        Utc.timestamp(metadata.header().timestamp() as i64, 0).to_rfc2822(),
        metadata.solid_entry_points().len(),
    );

    // The genesis transaction must be marked as SEP with snapshot index during loading a snapshot because coordinator
    // bootstraps the network by referencing the genesis tx.
    metadata.solid_entry_points.insert(MessageId::null());

    // node_builder = node_builder.with_worker_cfg::<worker::SnapshotWorker>(config.clone());

    Ok((node_builder, metadata))
}

pub fn events<N: Node>(_node: &N, _bus: Arc<Bus<'static>>) {
    // let snapshot_worker = node.worker::<worker::SnapshotWorker>().unwrap().tx.clone();
    //
    // bus.add_listener(move |latest_solid_milestone: &LatestSolidMilestoneChanged| {
    //     if let Err(e) = snapshot_worker.send(worker::SnapshotWorkerEvent(latest_solid_milestone.0.clone())) {
    //         warn!(
    //             "Failed to send milestone {} to snapshot worker: {:?}.",
    //             *latest_solid_milestone.0.index(),
    //             e
    //         )
    //     }
    // });
}
