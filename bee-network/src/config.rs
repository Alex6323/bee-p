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

use libp2p::Multiaddr;
use serde::Deserialize;

use std::str::FromStr;

const DEFAULT_BIND_ADDRESS: &str = "/ip4/0.0.0.0/tcp/15600";

pub const DEFAULT_MSG_BUFFER_SIZE: usize = 10000;
pub const DEFAULT_PEER_LIMIT: usize = 8;
pub const DEFAULT_RECONNECT_MILLIS: u64 = 60000;

/// Network configuration builder.
#[derive(Default, Deserialize)]
pub struct NetworkConfigBuilder {
    bind_address: Option<Multiaddr>,
    msg_buffer_size: Option<usize>,
    peer_limit: Option<usize>,
    reconnect_millis: Option<u64>,
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

    pub fn msg_buffer_size(mut self, msg_buffer_size: usize) -> Self {
        self.msg_buffer_size.replace(msg_buffer_size);
        self
    }

    pub fn peer_limit(mut self, peer_limit: usize) -> Self {
        self.peer_limit.replace(peer_limit);
        self
    }

    pub fn reconnect_millis(mut self, reconnect_millis: u64) -> Self {
        self.reconnect_millis.replace(reconnect_millis);
        self
    }

    /// Builds the network config.
    pub fn finish(self) -> NetworkConfig {
        NetworkConfig {
            bind_address: self
                .bind_address
                .unwrap_or(Multiaddr::from_str(DEFAULT_BIND_ADDRESS).unwrap()),
            msg_buffer_size: self.msg_buffer_size.unwrap_or(DEFAULT_MSG_BUFFER_SIZE),
            peer_limit: self.peer_limit.unwrap_or(DEFAULT_PEER_LIMIT),
            reconnect_millis: self.reconnect_millis.unwrap_or(DEFAULT_RECONNECT_MILLIS),
        }
    }
}

#[derive(Clone, Debug)]
pub struct NetworkConfig {
    pub bind_address: Multiaddr,
    pub msg_buffer_size: usize,
    pub peer_limit: usize,
    pub reconnect_millis: u64,
}

impl NetworkConfig {
    pub fn build() -> NetworkConfigBuilder {
        NetworkConfigBuilder::new()
    }
}
