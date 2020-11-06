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

use libp2p::{identity::Keypair, Multiaddr};
use serde::Deserialize;

use std::str::FromStr;

const DEFAULT_BIND_ADDRESS: &str = "/ip4/0.0.0.0/tcp/15600";

pub const DEFAULT_MAX_BUFFER_SIZE: usize = 10000;
pub const DEFAULT_RECONNECT_INTERVAL: u64 = 60;

/// Network configuration builder.
#[derive(Default, Deserialize)]
pub struct NetworkConfigBuilder {
    bind_address: Option<Multiaddr>,
    max_buffer_size: Option<usize>,
    reconnect_interval: Option<u64>,
}

impl NetworkConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind_address(mut self, bind_address: &str) -> Self {
        self.bind_address
            .replace(Multiaddr::from_str(bind_address).unwrap_or_else(|e| panic!("Error parsing address: {:?}", e)));
        self
    }

    pub fn reconnect_interval(mut self, interval: u64) -> Self {
        self.reconnect_interval.replace(interval);
        self
    }

    pub fn max_buffer_size(mut self, max_buffer_size: usize) -> Self {
        self.max_buffer_size.replace(max_buffer_size);
        self
    }

    /// Builds the network config.
    pub fn finish(self) -> NetworkConfig {
        NetworkConfig {
            bind_address: self
                .bind_address
                .unwrap_or(Multiaddr::from_str(DEFAULT_BIND_ADDRESS).unwrap()),
            max_buffer_size: self.max_buffer_size.unwrap_or(DEFAULT_MAX_BUFFER_SIZE),
            reconnect_interval: self.reconnect_interval.unwrap_or(DEFAULT_RECONNECT_INTERVAL),
        }
    }
}

#[derive(Clone, Debug)]
pub struct NetworkConfig {
    pub bind_address: Multiaddr,
    pub max_buffer_size: usize,
    pub reconnect_interval: u64,
}

impl NetworkConfig {
    pub fn build() -> NetworkConfigBuilder {
        NetworkConfigBuilder::new()
    }
}
