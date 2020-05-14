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

// TODO add acceptAnyConnection

const DEFAULT_LIMIT: u8 = 5;
const DEFAULT_PEERS: Vec<String> = Vec::new();

#[derive(Default, Deserialize)]
pub struct StaticPeeringConfigBuilder {
    pub(crate) limit: Option<u8>,
    pub(crate) peers: Option<Vec<String>>,
}

impl StaticPeeringConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn limit(mut self, limit: u8) -> Self {
        self.limit.replace(limit);
        self
    }

    pub fn add_peer(mut self, peer: &str) {
        if self.peers.is_none() {
            self.peers.replace(Vec::new());
        }
        self.peers.unwrap().push(peer.to_owned());
    }

    pub fn build(self) -> StaticPeeringConfig {
        StaticPeeringConfig {
            limit: self.limit.unwrap_or(DEFAULT_LIMIT),
            peers: self.peers.unwrap_or(DEFAULT_PEERS),
        }
    }
}

#[derive(Clone)]
pub struct StaticPeeringConfig {
    pub(crate) limit: u8,
    pub(crate) peers: Vec<String>,
}
