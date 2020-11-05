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

use libp2p::{Multiaddr, PeerId};

use dashmap::{mapref::entry::Entry, DashMap};

use std::sync::Arc;

const DEFAULT_KNOWN_PEERLIST_CAPACITY: usize = 16;

#[derive(Clone, Debug)]
pub struct KnownPeerList(Arc<DashMap<Multiaddr, Option<PeerId>>>);

impl KnownPeerList {
    pub fn new() -> Self {
        Self(Arc::new(DashMap::with_capacity(DEFAULT_KNOWN_PEERLIST_CAPACITY)))
    }

    pub fn insert_address(&self, address: Multiaddr) -> bool {
        match self.0.entry(address) {
            Entry::Occupied(_) => false,
            Entry::Vacant(entry) => {
                entry.insert(None);
                true
            }
        }
    }

    pub fn insert_full(&self, address: Multiaddr, peer_id: PeerId) -> bool {
        match self.0.entry(address) {
            Entry::Occupied(_) => false,
            Entry::Vacant(entry) => {
                entry.insert(Some(peer_id));
                true
            }
        }
    }

    pub fn update_peer_id(&self, address: Multiaddr, peer_id: PeerId) -> bool {
        if !self.contains_address(&address) {
            return false;
        }
        let _ = self.0.get_mut(&address).unwrap().replace(peer_id);
        true
    }

    pub fn contains_address(&self, address: &Multiaddr) -> bool {
        self.0.contains_key(address)
    }

    pub fn contains_peer_id(&self, peer_id: &PeerId) -> bool {
        self.0.iter().any(|kv| {
            if let Some(other_id) = kv.value().as_ref() {
                other_id == peer_id
            } else {
                false
            }
        })
    }

    pub fn remove_peer_by_address(&self, address: &Multiaddr) -> Option<Option<PeerId>> {
        self.0.remove(address).map(|kv| kv.1)
    }

    pub fn remove_peer_by_id(&self, peer_id: &PeerId) -> Option<Multiaddr> {
        let found = self.0.iter().find_map(|kv| {
            if let Some(other_id) = kv.value().as_ref() {
                if other_id == peer_id {
                    Some(kv.key().clone())
                } else {
                    None
                }
            } else {
                None
            }
        });
        if let Some(address) = found.as_ref() {
            let _ = self.0.remove(address);
        }
        found
    }

    // pub fn get_peer_id_from_address(&self, address: &Multiaddr) -> Option<PeerId> {
    //     self.0
    //         .get(address)
    //         .map(|kv| {
    //             if let Some(peer_id) = kv.value().as_ref() {
    //                 Some(peer_id.clone())
    //             } else {
    //                 None
    //             }
    //         })
    //         .flatten()
    // }

    pub fn get_address_from_peer_id(&self, peer_id: &PeerId) -> Option<Multiaddr> {
        self.0.iter().find_map(|kv| {
            if let Some(other_id) = kv.value().as_ref() {
                if other_id == peer_id {
                    Some(kv.key().clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        self.0.len()
    }
}
