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

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub struct ConfigBuilder {
    binding_address: Option<SocketAddr>,
    peers: Vec<String>,
    message: Option<String>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            binding_address: None,
            peers: vec![],
            message: None,
        }
    }

    pub fn with_binding_address(mut self, binding_address: SocketAddr) -> Self {
        self.binding_address.replace(binding_address);
        self
    }

    pub fn with_peer_url(mut self, peer_url: String) -> Self {
        self.peers.push(peer_url);
        self
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message.replace(message);
        self
    }

    pub fn finish(self) -> Config {
        Config {
            binding_address: self
                .binding_address
                .unwrap_or(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1337)),
            peers: self.peers,
            message: self.message.unwrap_or("hello".into()),
        }
    }
}

#[derive(Clone)]
pub struct Config {
    pub binding_address: SocketAddr,
    pub peers: Vec<String>,
    pub message: String,
}

impl Config {
    pub fn build() -> ConfigBuilder {
        ConfigBuilder::new()
    }
}
