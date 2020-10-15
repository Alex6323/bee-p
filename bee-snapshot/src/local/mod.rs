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
mod file;

pub(crate) use download::{download_local_snapshot, Error as DownloadError};

pub use config::{LocalSnapshotConfig, LocalSnapshotConfigBuilder};
pub use file::Error as FileError;

use crate::{header::SnapshotHeader, metadata::SnapshotMetadata};

use log::{error, info};

use std::collections::HashMap;

pub struct LocalSnapshot {
    pub(crate) metadata: SnapshotMetadata,
    pub(crate) state: HashMap<Address, u64>,
}

impl LocalSnapshot {
    pub fn metadata(&self) -> &SnapshotMetadata {
        &self.metadata
    }

    pub fn state(&self) -> &HashMap<Address, u64> {
        &self.state
    }
}

#[derive(Debug)]
pub(crate) enum Error {}

#[allow(dead_code)] // TODO: When pruning is enabled
pub(crate) fn snapshot(path: &str, index: u32) -> Result<(), Error> {
    info!("Creating local snapshot at index {}...", index);

    let ls = LocalSnapshot {
        metadata: SnapshotMetadata {
            header: SnapshotHeader {
                coordinator: Hash::zeros(),
                hash: Hash::zeros(),
                snapshot_index: index,
                entry_point_index: index,
                pruning_index: index,
                timestamp: 0,
            },
            solid_entry_points: HashMap::new(),
        },
        state: HashMap::new(),
    };

    let file = path.to_string() + "_tmp";

    if let Err(e) = ls.to_file(&file) {
        error!("Failed to write local snapshot to file {}: {:?}.", file, e);
    }

    info!("Created local snapshot at index {}.", index);

    Ok(())
}
