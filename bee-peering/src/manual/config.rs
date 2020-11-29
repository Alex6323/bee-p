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

use bee_network::Multiaddr;

use std::str::FromStr;

// TODO add acceptAnyConnection

const DEFAULT_LIMIT: u8 = 5;

#[derive(Default, Deserialize)]
pub struct ManualPeeringConfigBuilder {
    pub(crate) limit: Option<u8>,
    pub(crate) peers: Vec<(String, Option<String>)>,
}

impl ManualPeeringConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn limit(mut self, limit: u8) -> Self {
        self.limit.replace(limit);
        self
    }

    pub fn add_peer(mut self, multiaddr: &str, alias: Option<&str>) {
        self.peers.push((multiaddr.to_owned(), alias.map(|s| s.to_owned())));
    }

    pub fn finish(self) -> ManualPeeringConfig {
        let peers = self
            .peers
            .into_iter()
            .map(|peer| {
                (
                    Multiaddr::from_str(&peer.0[..]).expect("error parsing multiaddr."),
                    peer.1,
                )
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
    pub(crate) peers: Vec<(Multiaddr, Option<String>)>,
}

impl ManualPeeringConfig {
    pub fn build() -> ManualPeeringConfigBuilder {
        ManualPeeringConfigBuilder::new()
    }
}
