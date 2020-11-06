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

use crate::peer::Peer;

use bee_network::PeerId;

use dashmap::DashMap;
use tokio::sync::RwLock;

use std::sync::Arc;

pub(crate) struct PeerManager {
    pub(crate) peers: DashMap<PeerId, Arc<Peer>>,
    pub(crate) peers_keys: RwLock<Vec<PeerId>>,
}

impl PeerManager {
    pub(crate) fn new() -> Self {
        Self {
            peers: Default::default(),
            peers_keys: Default::default(),
        }
    }

    pub(crate) fn add(&self, peer: Arc<Peer>) {
        self.peers.insert(peer.id.clone(), peer);
    }

    pub(crate) async fn remove(&self, id: &PeerId) {
        self.peers.remove(id);
        self.peers_keys.write().await.retain(|peer_id| peer_id != id);
    }

    pub(crate) fn connected_peers(&self) -> u8 {
        // TODO impl
        0
    }

    pub(crate) fn synced_peers(&self) -> u8 {
        // TODO impl
        0
    }
}
