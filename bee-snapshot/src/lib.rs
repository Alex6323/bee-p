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
pub mod global;
pub mod header;
pub mod local;
pub mod metadata;

use global::GlobalSnapshot;
use header::SnapshotHeader;
use local::LocalSnapshot;
use metadata::SnapshotMetadata;

use bee_common_ext::{event::Bus, node::Node};
use bee_crypto::ternary::Hash;
use bee_transaction::bundled::Address;
// use bee_protocol::{event::LatestSolidMilestoneChanged, MilestoneIndex};

use chrono::{offset::TimeZone, Utc};
use log::info;

use std::{collections::HashMap, path::Path, sync::Arc};

#[derive(Debug)]
pub enum Error {
    Global(global::FileError),
    Local(local::FileError),
    Download(local::DownloadError),
}

// TODO change return type

pub async fn init<N: Node>(
    // tangle: &MsTangle<B>,
    config: &config::SnapshotConfig,
    node_builder: N::Builder,
) -> Result<(N::Builder, HashMap<Address, u64>, SnapshotMetadata), Error> {
    let (state, mut metadata) = match config.load_type() {
        config::LoadType::Global => {
            info!("Loading global snapshot file {}...", config.global().path());

            let snapshot = global::GlobalSnapshot::from_file(config.global().path(), *config.global().index())
                .map_err(Error::Global)?;

            info!(
                "Loaded global snapshot file from with index {} and {} balances.",
                *config.global().index(),
                snapshot.state().len()
            );

            let GlobalSnapshot { state, index } = snapshot;

            let metadata = SnapshotMetadata {
                header: SnapshotHeader {
                    coordinator: Hash::zeros(),
                    hash: Hash::zeros(),
                    snapshot_index: index,
                    entry_point_index: index,
                    pruning_index: index,
                    // TODO from conf ?
                    timestamp: 0,
                },
                solid_entry_points: HashMap::new(),
                seen_milestones: HashMap::new(),
            };

            (state, metadata)
        }
        config::LoadType::Local => {
            if !Path::new(config.local().path()).exists() {
                local::download_local_snapshot(config.local())
                    .await
                    .map_err(Error::Download)?;
            }
            info!("Loading local snapshot file {}...", config.local().path());

            let snapshot = local::LocalSnapshot::from_file(config.local().path()).map_err(Error::Local)?;

            info!(
                "Loaded local snapshot file from {} with index {}, {} solid entry points, {} seen milestones and \
                {} balances.",
                Utc.timestamp(snapshot.metadata().timestamp() as i64, 0).to_rfc2822(),
                snapshot.metadata().index(),
                snapshot.metadata().solid_entry_points().len(),
                snapshot.metadata().seen_milestones().len(),
                snapshot.state.len()
            );

            let LocalSnapshot { metadata, state } = snapshot;

            (state, metadata)
        }
    };

    // The genesis transaction must be marked as SEP with snapshot index during loading a global snapshot
    // because coordinator bootstraps the network by referencing the genesis tx.
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
