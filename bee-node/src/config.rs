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

use bee_common::logger::{LoggerConfig, LoggerConfigBuilder};
use bee_network::{NetworkConfig, NetworkConfigBuilder};
use bee_peering::{PeeringConfig, PeeringConfigBuilder};
use bee_protocol::{ProtocolConfig, ProtocolConfigBuilder};
use bee_snapshot::{SnapshotConfig, SnapshotConfigBuilder};

use serde::Deserialize;

use std::fs;

const CONFIG_PATH: &str = "./config.toml";

// TODO use proper error
/// Creates a Bee config builder from the local config file.
pub fn read_config() -> Result<NodeConfigBuilder, ()> {
    match fs::read_to_string(CONFIG_PATH) {
        Ok(toml) => match toml::from_str::<NodeConfigBuilder>(&toml) {
            Ok(config_builder) => Ok(config_builder),
            Err(_) => Err(()),
        },
        Err(_) => Err(()),
    }
}

#[derive(Default, Deserialize)]
pub struct NodeConfigBuilder {
    pub(crate) logger: LoggerConfigBuilder,
    pub(crate) network: NetworkConfigBuilder,
    pub(crate) peering: PeeringConfigBuilder,
    pub(crate) protocol: ProtocolConfigBuilder,
    pub(crate) snapshot: SnapshotConfigBuilder,
}

impl NodeConfigBuilder {
    pub fn finish(self) -> NodeConfig {
        NodeConfig {
            logger: self.logger.finish(),
            network: self.network.finish(),
            peering: self.peering.finish(),
            protocol: self.protocol.finish(),
            snapshot: self.snapshot.finish(),
        }
    }
}

#[derive(Clone)]
pub struct NodeConfig {
    pub logger: LoggerConfig,
    pub network: NetworkConfig,
    pub peering: PeeringConfig,
    pub protocol: ProtocolConfig,
    pub snapshot: SnapshotConfig,
}
