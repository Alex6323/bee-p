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
// pub(crate) mod worker;

pub mod config;
pub mod event;
pub mod header;
pub mod local;
pub mod metadata;

use local::LocalSnapshot;
use metadata::SnapshotMetadata;

use bee_common_ext::{event::Bus, node::Node};
// use bee_protocol::{event::LatestSolidMilestoneChanged, MilestoneIndex};

use chrono::{offset::TimeZone, Utc};
use log::info;

use std::{collections::HashMap, path::Path, sync::Arc};

#[derive(Debug)]
pub enum Error {
    Local(local::FileError),
    Download(local::DownloadError),
}

// TODO change return type

pub async fn init<N: Node>(
    // tangle: &MsTangle<B>,
    config: &config::SnapshotConfig,
    node_builder: N::Builder,
) -> Result<(N::Builder, HashMap<Address, u64>, SnapshotMetadata), Error> {
    if !Path::new(config.local().path()).exists() {
        local::download_local_snapshot(config.local())
            .await
            .map_err(Error::Download)?;
    }
    info!("Loading local snapshot file {}...", config.local().path());

    let LocalSnapshot { metadata, state } =
        local::LocalSnapshot::from_file(config.local().path()).map_err(Error::Local)?;

    info!(
        "Loaded local snapshot file from {} with index {}, {} solid entry points, {} seen milestones and \
                {} balances.",
        Utc.timestamp(metadata.timestamp() as i64, 0).to_rfc2822(),
        metadata.index(),
        metadata.solid_entry_points().len(),
        metadata.seen_milestones().len(),
        state.len()
    );

    // The genesis transaction must be marked as SEP with snapshot index during loading a snapshot because coordinator
    // bootstraps the network by referencing the genesis tx.
    metadata.solid_entry_points.insert(Hash::zeros(), metadata.index());

    // node_builder = node_builder.with_worker_cfg::<worker::SnapshotWorker>(config.clone());

    Ok((node_builder, state, metadata))
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
