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

#[derive(Default, Deserialize)]
pub struct LocalSnapshotConfigBuilder {
    path: Option<String>,
    download_urls: Option<Vec<String>>,
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

    pub fn finish(self) -> LocalSnapshotConfig {
        LocalSnapshotConfig {
            path: self.path.unwrap_or_else(|| DEFAULT_PATH.to_string()),
            download_urls: self.download_urls.unwrap_or_else(|| DEFAULT_DOWNLOAD_URLS),
        }
    }
}

#[derive(Clone)]
pub struct LocalSnapshotConfig {
    path: String,
    download_urls: Vec<String>,
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
}
