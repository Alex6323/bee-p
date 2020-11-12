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

use bee_network::{Multiaddr, MultiaddrPeerId, PeerId};

use std::str::FromStr;

// TODO add acceptAnyConnection

const DEFAULT_LIMIT: u8 = 5;

#[derive(Default, Deserialize)]
pub struct ManualPeeringConfigBuilder {
    pub(crate) limit: Option<u8>,
    pub(crate) peers: Vec<String>,
}

impl ManualPeeringConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn limit(mut self, limit: u8) -> Self {
        self.limit.replace(limit);
        self
    }

    pub fn add_peer(mut self, peer_address_id: &str) {
        self.peers.push(peer_address_id.to_owned());
    }

    pub fn finish(self) -> ManualPeeringConfig {
        let peers = self
            .peers
            .iter()
            .map(|s| {
                MultiaddrPeerId::from_str(s)
                    .expect("error parsing MultiaddrPeerId")
                    .split()
            })
            .collect();

        ManualPeeringConfig {
            limit: self.limit.unwrap_or(DEFAULT_LIMIT),
            peers,
        }
    }
}

#[derive(Clone)]
pub struct ManualPeeringConfig {
    pub(crate) limit: u8,
    pub(crate) peers: Vec<(Multiaddr, PeerId)>,
}

impl ManualPeeringConfig {
    pub fn build() -> ManualPeeringConfigBuilder {
        ManualPeeringConfigBuilder::new()
    }
}
