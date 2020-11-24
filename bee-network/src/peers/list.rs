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

use crate::ReadableId;

use super::{errors::Error, DataSender};

use dashmap::{mapref::entry::Entry, DashMap};
use libp2p::PeerId;

use std::sync::Arc;

const DEFAULT_PEERLIST_CAPACITY: usize = 8;

#[derive(Clone, Debug, Default)]
pub struct PeerList(Arc<DashMap<PeerId, Option<DataSender>>>);

impl PeerList {
    pub fn new() -> Self {
        Self(Arc::new(DashMap::with_capacity(DEFAULT_PEERLIST_CAPACITY)))
    }

    pub fn insert_peer(&self, id: PeerId) -> bool {
        match self.0.entry(id) {
            Entry::Occupied(_) => false,
            Entry::Vacant(entry) => {
                entry.insert(None);
                true
            }
        }
    }

    pub fn insert_connected_peer(&self, id: PeerId, sender: DataSender) -> bool {
        match self.0.entry(id) {
            Entry::Occupied(mut entry) => entry.get_mut().replace(sender).is_none(),
            Entry::Vacant(entry) => {
                entry.insert(Some(sender));
                true
            }
        }
    }

    pub fn contains_peer(&self, id: &PeerId) -> bool {
        self.0.contains_key(id)
    }

    pub fn remove_peer(&self, id: &PeerId) -> bool {
        self.0.remove(id).is_some()
    }

    pub fn remove_peer_connection(&self, id: &PeerId) -> bool {
        self.0
            .get_mut(id)
            .and_then(|mut kv| {
                std::mem::swap(kv.value_mut(), &mut None);
                Some(())
            })
            .is_some()
    }

    pub async fn send_message(&mut self, message: Vec<u8>, to: &PeerId) -> Result<(), Error> {
        if let Some(mut kv) = self.0.get_mut(to) {
            if let Some(sender) = kv.value_mut() {
                sender
                    .send_async(message)
                    .await
                    .map_err(|_| Error::SendMessageFailure(to.readable()))?;

                Ok(())
            } else {
                Err(Error::DisconnectedPeer(to.readable()))
            }
        } else {
            Err(Error::UnknownPeer(to.readable()))
        }
    }

    pub fn num_peers(&self) -> usize {
        self.0.len()
    }

    pub fn num_connected(&self) -> usize {
        self.0
            .iter()
            .fold(0, |count, kv| if kv.value().is_some() { count + 1 } else { count })
    }
}
