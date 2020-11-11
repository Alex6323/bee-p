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

use libp2p::PeerId;

use std::collections::{hash_map::Entry, HashMap};

#[derive(Clone, Debug)]
pub struct ConnectedPeer {
    data_sender: DataSender,
    // duplicate_of: Option<PeerId>,
}
#[derive(Default)]
pub struct ConnectedPeerList(HashMap<PeerId, ConnectedPeer>);

impl ConnectedPeerList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, peer_id: PeerId, data_sender: DataSender) -> bool {
        match self.0.entry(peer_id) {
            Entry::Occupied(_) => false,
            Entry::Vacant(entry) => {
                entry.insert(ConnectedPeer {
                    data_sender,
                    // duplicate_of: None,
                });
                true
            }
        }
    }

    pub fn contains(&self, peer_id: PeerId) -> bool {
        self.0.contains_key(&peer_id)
    }

    pub fn remove(&mut self, peer_id: &PeerId) -> bool {
        // assert!(!(self.is_duplicate(peer_id) && self.has_duplicate(peer_id).is_some()));

        // // NOTE: here we are removing the original before the duplicate. Should we deny that?
        // if let Some(duplicate_epid) = self.has_duplicate(peer_id) {
        //     self.0.get_mut(&duplicate_epid).map(|v| v.duplicate_of.take());
        // }

        self.0.remove(peer_id).is_some()
    }

    // pub fn mark_duplicate(&mut self, duplicate_epid: PeerId, original_epid: PeerId) -> bool {
    //     self.0.get_mut(&duplicate_epid).map_or(false, |endpoint| {
    //         endpoint.duplicate_of.replace(original_epid);
    //         true
    //     })
    // }

    // pub fn has_duplicate(&self, peer_id: PeerId) -> Option<PeerId> {
    //     self.0
    //         .iter()
    //         .find(|(_, endpoint)| endpoint.duplicate_of.map_or(false, |other| other == epid))
    //         .map(|(duplicate, _)| *duplicate)
    // }

    // pub fn is_duplicate(&self, epid: PeerId) -> bool {
    //     self.0.get(&epid).map_or(false, |v| v.duplicate_of.is_some())
    // }

    pub async fn send_message(&mut self, message: Vec<u8>, peer_id: PeerId) -> Result<bool, WorkerError> {
        if let Some(connected_endpoint) = self.0.get_mut(&peer_id) {
            connected_endpoint
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
