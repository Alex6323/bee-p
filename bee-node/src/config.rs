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
use bee_protocol::config::{ProtocolConfig, ProtocolConfigBuilder};
use bee_snapshot::config::{SnapshotConfig, SnapshotConfigBuilder};
use bee_storage::storage::Backend;

use blake2::{
    digest::{Update, VariableOutput},
    VarBlake2b,
};
use serde::Deserialize;
use thiserror::Error;

use std::{convert::TryInto, fs, path::Path};

const DEFAULT_NETWORK_ID: &str = "alphanet1";

#[derive(Debug, Error)]
pub enum Error {
    #[error("Reading the specified config file failed: {0}.")]
    ConfigFileReadFailure(#[from] std::io::Error),

    #[error("Deserializing the node config builder failed: {0}.")]
    NodeConfigBuilderCreationFailure(#[from] toml::de::Error),
}

#[derive(Default, Deserialize)]
pub struct NodeConfigBuilder<B: Backend> {
    pub(crate) network_id: Option<String>,
    pub(crate) logger: LoggerConfigBuilder,
    pub(crate) network: NetworkConfigBuilder,
    pub(crate) peering: PeeringConfigBuilder,
    pub(crate) protocol: ProtocolConfigBuilder,
    pub(crate) snapshot: SnapshotConfigBuilder,
    pub(crate) database: B::ConfigBuilder,
}

impl<B: Backend> NodeConfigBuilder<B> {
    /// Creates a node config builder from a local config file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        match fs::read_to_string(path) {
            Ok(toml) => toml::from_str::<Self>(&toml).map_err(Error::NodeConfigBuilderCreationFailure),
            Err(e) => Err(Error::ConfigFileReadFailure(e)),
        }
    }

    pub fn finish(self) -> NodeConfig<B> {
        let mut hasher = VarBlake2b::new(32).unwrap();
        let mut network_id: (String, u64) = (self.network_id.unwrap_or_else(|| DEFAULT_NETWORK_ID.to_string()), 0);
        hasher.update(network_id.0.as_bytes());
        hasher.finalize_variable(|res| network_id.1 = u64::from_le_bytes(res[0..8].try_into().unwrap()));

        NodeConfig {
            network_id,
            logger: self.logger.finish(),
            network: self.network.finish(),
            peering: self.peering.finish(),
            protocol: self.protocol.finish(),
            snapshot: self.snapshot.finish(),
            database: self.database.into(),
        }
    }
}

#[derive(Clone)]
pub struct NodeConfig<B: Backend> {
    pub network_id: (String, u64),
    pub logger: LoggerConfig,
    pub network: NetworkConfig,
    pub peering: PeeringConfig,
    pub protocol: ProtocolConfig,
    pub snapshot: SnapshotConfig,
    pub database: B::Config,
}
