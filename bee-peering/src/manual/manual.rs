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

use crate::{manual::config::ManualPeeringConfig, PeerManager};

use bee_network::{Command::AddEndpoint, Network};

use async_trait::async_trait;
use log::warn;

// Manages a peer list and watches a config file for changes
// Sends changes (peer added/removed) to the network

pub struct ManualPeerManager {
    config: ManualPeeringConfig,
    network: Network,
}

impl ManualPeerManager {
    pub fn new(config: ManualPeeringConfig, network: Network) -> Self {
        Self { config, network }
    }

    fn add_endpoint(&mut self, url: &str) {
        if let Err(e) = self.network.unbounded_send(AddEndpoint { url: url.to_string() }) {
            warn!("Failed to add endpoint \"{}\": {}", url, e);
        }
    }
}

#[async_trait]
impl PeerManager for ManualPeerManager {
    async fn run(mut self) {
        // TODO config file watcher
        // TODO use limit
        for peer in self.config.peers.clone() {
            self.add_endpoint(&peer);
        }
    }
}
