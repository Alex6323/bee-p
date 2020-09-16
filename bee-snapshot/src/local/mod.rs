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

mod config;
mod download;
mod metadata;
mod snapshot;

pub(crate) use download::{download_local_snapshot, Error as DownloadError};

pub use config::{LocalSnapshotConfig, LocalSnapshotConfigBuilder};
pub use metadata::LocalSnapshotMetadata;
pub use snapshot::{Error, LocalSnapshot};

use crate::metadata::SnapshotMetadata;

use bee_crypto::ternary::Hash;
use bee_ledger::state::LedgerState;

use log::{error, info};

use std::collections::HashMap;

pub(crate) fn snapshot(path: &str, index: u32) {
    info!("Creating local snapshot at index {}...", index);

    let ls = LocalSnapshot {
        metadata: LocalSnapshotMetadata {
            inner: SnapshotMetadata {
                coordinator: Hash::zeros(),
                hash: Hash::zeros(),
                snapshot_index: index,
                entry_point_index: index,
                pruning_index: index,
                timestamp: 0,
            },
            solid_entry_points: HashMap::new(),
            seen_milestones: HashMap::new(),
        },
        state: LedgerState::new(),
    };

    let file = path.to_string() + "_tmp";

    if let Err(e) = ls.to_file(&file) {
        error!("Failed to write local snapshot to file {}.", file);
    }

    info!("Created local snapshot at index {}.", index);
}
