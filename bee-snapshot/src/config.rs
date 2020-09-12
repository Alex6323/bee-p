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

use crate::{
    global::{GlobalSnapshotConfig, GlobalSnapshotConfigBuilder},
    local::{LocalSnapshotConfig, LocalSnapshotConfigBuilder},
    pruning::{PruningConfig, PruningConfigBuilder},
};

use serde::Deserialize;

const DEFAULT_LOAD_TYPE: &str = "local";

#[derive(Clone)]
pub enum LoadType {
    Local,
    Global,
}

#[derive(Default, Deserialize)]
pub struct SnapshotConfigBuilder {
    load_type: Option<String>,
    local: LocalSnapshotConfigBuilder,
    global: GlobalSnapshotConfigBuilder,
    pruning: PruningConfigBuilder,
}

impl SnapshotConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn local_path(mut self, path: String) -> Self {
        self.local = self.local.path(path);
        self
    }

    pub fn global_path(mut self, path: String) -> Self {
        self.global = self.global.path(path);
        self
    }

    pub fn finish(self) -> SnapshotConfig {
        let load_type = match self.load_type.unwrap_or_else(|| DEFAULT_LOAD_TYPE.to_owned()).as_str() {
            "local" => LoadType::Local,
            "global" => LoadType::Global,
            _ => LoadType::Local,
        };

        SnapshotConfig {
            load_type,
            local: self.local.finish(),
            global: self.global.finish(),
            pruning: self.pruning.finish(),
        }
    }
}

#[derive(Clone)]
pub struct SnapshotConfig {
    load_type: LoadType,
    local: LocalSnapshotConfig,
    global: GlobalSnapshotConfig,
    pruning: PruningConfig,
}

impl SnapshotConfig {
    pub fn build() -> SnapshotConfigBuilder {
        SnapshotConfigBuilder::new()
    }

    pub fn load_type(&self) -> &LoadType {
        &self.load_type
    }

    pub fn local(&self) -> &LocalSnapshotConfig {
        &self.local
    }

    pub fn global(&self) -> &GlobalSnapshotConfig {
        &self.global
    }

    pub fn pruning(&self) -> &PruningConfig {
        &self.pruning
    }
}
