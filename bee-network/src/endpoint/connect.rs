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

use super::{DataSender, EndpointId};

use bee_common::worker::Error as WorkerError;

use futures::sink::SinkExt;

use std::collections::{hash_map::Entry, HashMap};

#[derive(Clone, Debug)]
pub struct ConnectedEndpoint {
    data_sender: DataSender,
    duplicate_of: Option<EndpointId>,
}
#[derive(Default)]
pub struct ConnectedEndpointList(HashMap<EndpointId, ConnectedEndpoint>);

impl ConnectedEndpointList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, epid: EndpointId, data_sender: DataSender) -> bool {
        match self.0.entry(epid) {
            Entry::Occupied(_) => false,
            Entry::Vacant(entry) => {
                entry.insert(ConnectedEndpoint {
                    data_sender,
                    duplicate_of: None,
                });
                true
            }
        }
    }

    pub fn contains(&self, epid: EndpointId) -> bool {
        self.0.contains_key(&epid)
    }

    pub fn remove(&mut self, epid: EndpointId) -> bool {
        assert!(!(self.is_duplicate(epid) && self.has_duplicate(epid).is_some()));

        // NOTE: here we are removing the original before the duplicate. Should we deny that?
        if let Some(duplicate_epid) = self.has_duplicate(epid) {
            self.0.get_mut(&duplicate_epid).map(|v| v.duplicate_of.take());
        }

        self.0.remove(&epid).is_some()
    }

    pub fn mark_duplicate(&mut self, duplicate_epid: EndpointId, original_epid: EndpointId) -> bool {
        self.0.get_mut(&duplicate_epid).map_or(false, |endpoint| {
            endpoint.duplicate_of.replace(original_epid);
            true
        })
    }

    pub fn has_duplicate(&self, epid: EndpointId) -> Option<EndpointId> {
        self.0
            .iter()
            .find(|(_, endpoint)| endpoint.duplicate_of.map_or(false, |other| other == epid))
            .map_or(None, |(duplicate, _)| Some(*duplicate))
    }

    pub fn is_duplicate(&self, epid: EndpointId) -> bool {
        self.0.get(&epid).map_or(false, |v| v.duplicate_of.is_some())
    }

    pub async fn send_message(&mut self, message: Vec<u8>, epid: EndpointId) -> Result<bool, WorkerError> {
        if let Some(connected_endpoint) = self.0.get_mut(&epid) {
            connected_endpoint.data_sender.send(message).await?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
