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

use std::{
    collections::{hash_map::Entry, HashMap},
    net::SocketAddr,
};

#[derive(Clone, Debug)]
pub(crate) struct ConnectedEndpoint {
    epid: EndpointId,
    socket_address: SocketAddr,
    connected_timestamp: u64, // NOTE: probably not necessary
    sender: DataSender,
    duplicate_of: Option<EndpointId>,
}
#[derive(Default)]
pub(crate) struct ConnectedEndpointList(HashMap<EndpointId, ConnectedEndpoint>);

impl ConnectedEndpointList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(
        &mut self,
        epid: EndpointId,
        socket_address: SocketAddr,
        connected_timestamp: u64,
        sender: DataSender,
    ) -> bool {
        match self.0.entry(epid) {
            Entry::Occupied(_) => false,
            Entry::Vacant(entry) => {
                entry.insert(ConnectedEndpoint {
                    epid,
                    socket_address,
                    connected_timestamp,
                    sender,
                    duplicate_of: None,
                });
                true
            }
        }
    }

    pub fn contains(&self, epid: &EndpointId) -> bool {
        self.0.contains_key(epid)
    }

    pub fn remove(&mut self, epid: &EndpointId) -> bool {
        assert!(!(self.is_duplicate(epid) && self.has_duplicate(epid).is_some()));

        // NOTE: here we are removing the original before the duplicate. Should we deny that?
        if let Some(duplicate) = self.has_duplicate(epid) {
            self.0.get_mut(duplicate).map(|v| v.duplicate_of.take());
        }

        self.0.remove(epid).is_some()
    }

    pub fn set_duplicate(&mut self, duplicate: EndpointId, duplicate_of: EndpointId) -> bool {
        self.0.get_mut(&duplicate).map_or(false, |endpoint| {
            endpoint.duplicate_of.replace(duplicate_of);
            true
        })
    }

    pub fn has_duplicate(&self, epid: &EndpointId) -> Option<&EndpointId> {
        self.0
            .iter()
            .find(|(_, endpoint)| endpoint.duplicate_of.map_or(false, |ref other| other == epid))
            .map_or(None, |(duplicate, _)| Some(duplicate))
    }

    pub fn is_duplicate(&self, epid: &EndpointId) -> bool {
        self.0.get(epid).map_or(false, |v| v.duplicate_of.is_some())
    }

    pub async fn send(&mut self, data: Vec<u8>, epid: &EndpointId) -> Result<bool, WorkerError> {
        if let Some(connected_endpoint) = self.0.get_mut(epid) {
            connected_endpoint.sender.send(data).await?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
