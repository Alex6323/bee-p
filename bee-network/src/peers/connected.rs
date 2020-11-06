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

use super::DataSender;

use bee_common::worker::Error as WorkerError;

use dashmap::{mapref::entry::Entry, DashMap};
use libp2p::PeerId;

use std::sync::Arc;

const DEFAULT_CONNECTED_PEERLIST_CAPACITY: usize = 8;

#[derive(Clone, Debug)]
pub struct ConnectedPeer {
    data_sender: DataSender,
}
#[derive(Clone, Debug, Default)]
pub struct ConnectedPeerList(Arc<DashMap<PeerId, ConnectedPeer>>);

impl ConnectedPeerList {
    pub fn new() -> Self {
        Self(Arc::new(DashMap::with_capacity(DEFAULT_CONNECTED_PEERLIST_CAPACITY)))
    }

    pub fn insert(&mut self, peer_id: PeerId, data_sender: DataSender) -> bool {
        match self.0.entry(peer_id.clone()) {
            Entry::Occupied(_) => false,
            Entry::Vacant(entry) => {
                entry.insert(ConnectedPeer { data_sender });
                true
            }
        }
    }

    pub fn contains(&self, peer_id: &PeerId) -> bool {
        self.0.contains_key(peer_id)
    }

    pub fn remove(&mut self, peer_id: &PeerId) -> bool {
        self.0.remove(peer_id).is_some()
    }

    pub async fn send_message(&mut self, message: Vec<u8>, peer_id: &PeerId) -> Result<bool, WorkerError> {
        if let Some(connected_peer) = self.0.get_mut(peer_id) {
            connected_peer
                .data_sender
                .send_async(message)
                .await
                .map_err(|e| WorkerError(Box::new(e)))?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}
