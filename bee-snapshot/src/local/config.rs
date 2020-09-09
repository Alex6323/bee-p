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

use serde::Deserialize;

const DEFAULT_PATH: &str = "./snapshots/mainnet/export.bin";
const DEFAULT_DOWNLOAD_URLS: Vec<String> = Vec::new();
const DEFAULT_DEPTH: usize = 50;
const DEFAULT_INTERVAL_SYNCED: usize = 50;
const DEFAULT_INTERVAL_UNSYNCED: usize = 1000;

#[derive(Default, Deserialize)]
pub struct LocalSnapshotConfigBuilder {
    path: Option<String>,
    download_urls: Option<Vec<String>>,
    depth: Option<usize>,
    interval_synced: Option<usize>,
    interval_unsynced: Option<usize>,
}

impl LocalSnapshotConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn path(mut self, path: String) -> Self {
        self.path.replace(path);
        self
    }

    pub fn download_urls(mut self, download_urls: Vec<String>) -> Self {
        self.download_urls.replace(download_urls);
        self
    }

    pub fn depth(mut self, depth: usize) -> Self {
        self.depth.replace(depth);
        self
    }

    pub fn interval_synced(mut self, interval_synced: usize) -> Self {
        self.interval_synced.replace(interval_synced);
        self
    }

    pub fn interval_unsynced(mut self, interval_unsynced: usize) -> Self {
        self.interval_unsynced.replace(interval_unsynced);
        self
    }

    pub fn finish(self) -> LocalSnapshotConfig {
        LocalSnapshotConfig {
            path: self.path.unwrap_or_else(|| DEFAULT_PATH.to_string()),
            download_urls: self.download_urls.unwrap_or_else(|| DEFAULT_DOWNLOAD_URLS),
            depth: self.depth.unwrap_or(DEFAULT_DEPTH),
            interval_synced: self.interval_synced.unwrap_or(DEFAULT_INTERVAL_SYNCED),
            interval_unsynced: self.interval_unsynced.unwrap_or(DEFAULT_INTERVAL_UNSYNCED),
        }
    }
}

#[derive(Clone)]
pub struct LocalSnapshotConfig {
    path: String,
    download_urls: Vec<String>,
    depth: usize,
    interval_synced: usize,
    interval_unsynced: usize,
}

impl LocalSnapshotConfig {
    pub fn build() -> LocalSnapshotConfigBuilder {
        LocalSnapshotConfigBuilder::new()
    }

    pub fn path(&self) -> &String {
        &self.path
    }

    pub fn download_urls(&self) -> &Vec<String> {
        &self.download_urls
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn interval_synced(&self) -> usize {
        self.interval_synced
    }

    pub fn interval_unsynced(&self) -> usize {
        self.interval_unsynced
    }
}
