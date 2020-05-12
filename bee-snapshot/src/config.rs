// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use serde::Deserialize;

const DEFAULT_META_FILE_PATH: &str = "./data/mainnet.snapshot.meta";
const DEFAULT_STATE_FILE_PATH: &str = "./data/mainnet.snapshot.state";

#[derive(Default, Deserialize)]
pub struct SnapshotConfigBuilder {
    meta_file_path: Option<String>,
    state_file_path: Option<String>,
}

impl SnapshotConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn meta_file_path(mut self, meta_file_path: String) -> Self {
        self.meta_file_path.replace(meta_file_path);
        self
    }

    pub fn state_file_path(mut self, state_file_path: String) -> Self {
        self.state_file_path.replace(state_file_path);
        self
    }

    pub fn build(self) -> SnapshotConfig {
        SnapshotConfig {
            meta_file_path: self.meta_file_path.unwrap_or(DEFAULT_META_FILE_PATH.to_string()),
            state_file_path: self.state_file_path.unwrap_or(DEFAULT_STATE_FILE_PATH.to_string()),
        }
    }
}

#[derive(Clone)]
pub struct SnapshotConfig {
    meta_file_path: String,
    state_file_path: String,
}

impl SnapshotConfig {
    pub fn meta_file_path(&self) -> &String {
        &self.meta_file_path
    }

    pub fn state_file_path(&self) -> &String {
        &self.state_file_path
    }
}
