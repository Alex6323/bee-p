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

// TODO get peer info

use crate::peer::{HandshakedPeer, Peer};

use bee_network::{Multiaddr, PeerId};

use dashmap::DashMap;
use tokio::sync::RwLock;

use std::sync::Arc;

pub(crate) struct PeerManager {
    pub(crate) peers: DashMap<PeerId, Arc<Peer>>,
    pub(crate) handshaked_peers: DashMap<PeerId, Arc<HandshakedPeer>>,
    pub(crate) handshaked_peers_keys: RwLock<Vec<PeerId>>,
}

impl PeerManager {
    pub(crate) fn new() -> Self {
        Self {
            peers: Default::default(),
            handshaked_peers: Default::default(),
            handshaked_peers_keys: Default::default(),
        }
    }

    pub(crate) fn add(&self, peer: Arc<Peer>) {
        self.peers.insert(peer.id.clone(), peer);
    }

    pub(crate) async fn handshake(&self, id: &PeerId, address: Multiaddr) {
        if self.peers.remove(id).is_some() {
            // TODO check if not already added

            let peer = Arc::new(HandshakedPeer::new(id.clone(), address));

            self.handshaked_peers.insert(id.clone(), peer.clone());
            self.handshaked_peers_keys.write().await.push(id.clone());
        }
    }

    pub(crate) async fn remove(&self, id: &PeerId) {
        // TODO both ?
        self.peers.remove(id);

        self.handshaked_peers_keys.write().await.retain(|peer_id| peer_id != id);
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
