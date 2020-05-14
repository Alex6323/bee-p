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

use bee_network::{NetworkConfig, NetworkConfigBuilder};
use bee_peering::{PeeringConfig, PeeringConfigBuilder};
use bee_protocol::{ProtocolConfig, ProtocolConfigBuilder};
use bee_snapshot::{SnapshotConfig, SnapshotConfigBuilder};

use log;
use serde::Deserialize;

pub(crate) const CONFIG_PATH: &str = "./config.toml";
const DEFAULT_LOG_LEVEL: &str = "info";

#[derive(Default, Deserialize)]
pub struct NodeConfigBuilder {
    log_level: Option<String>,
    network: NetworkConfigBuilder,
    peering: PeeringConfigBuilder,
    protocol: ProtocolConfigBuilder,
    snapshot: SnapshotConfigBuilder,
}

impl NodeConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn log_level(mut self, log_level: &str) -> Self {
        self.log_level.replace(log_level.to_string());
        self
    }

    pub fn build(self) -> NodeConfig {
        let log_level = match self.log_level.unwrap_or(DEFAULT_LOG_LEVEL.to_owned()).as_str() {
            "trace" => log::LevelFilter::Trace,
            "debug" => log::LevelFilter::Debug,
            "info" => log::LevelFilter::Info,
            "warn" => log::LevelFilter::Warn,
            "error" => log::LevelFilter::Error,
            _ => log::LevelFilter::Info,
        };

        NodeConfig {
            log_level,
            network: self.network.build(),
            peering: self.peering.build(),
            protocol: self.protocol.build(),
            snapshot: self.snapshot.build(),
        }
    }
}

pub struct NodeConfig {
    pub(crate) log_level: log::LevelFilter,
    pub(crate) network: NetworkConfig,
    pub(crate) peering: PeeringConfig,
    pub(crate) protocol: ProtocolConfig,
    pub(crate) snapshot: SnapshotConfig,
}
