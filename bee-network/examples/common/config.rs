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

use bee_network::{Address, Url};

use async_std::task::block_on;

#[derive(Clone)]
pub struct Config {
    pub host_addr: Address,
    pub peers: Vec<Url>,
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }
}

pub struct ConfigBuilder {
    host_addr: Option<Address>,
    peers: Vec<Url>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            host_addr: None,
            peers: vec![],
        }
    }

    pub fn with_host_addr(mut self, host_addr: Address) -> Self {
        self.host_addr.replace(host_addr);
        self
    }

    pub fn with_peer_url(mut self, peer_url: Url) -> Self {
        self.peers.push(peer_url);
        self
    }

    pub fn build(self) -> Config {
        Config {
            host_addr: self
                .host_addr
                .unwrap_or_else(|| block_on(Address::from_addr_str("localhost:1337")).unwrap()),
            peers: self.peers,
        }
    }
}
