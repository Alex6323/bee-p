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

use crate::{address::Address, endpoint::EndpointId};

use bee_common::worker::Error as WorkerError;

use futures::{channel::mpsc, sink::SinkExt};

use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

const DATA_CHANNEL_CAPACITY: usize = 10000;

pub type DataSender = mpsc::Sender<Vec<u8>>;
pub type DataReceiver = mpsc::Receiver<Vec<u8>>;

pub fn channel() -> (DataSender, DataReceiver) {
    mpsc::channel(DATA_CHANNEL_CAPACITY)
}

#[derive(Clone, Debug)]
pub(crate) struct ConnectedEndpoint {
    epid: EndpointId,
    address: Address,
    timestamp: u64,
    data_sender: DataSender,
    duplicate: Option<EndpointId>,
}
#[derive(Default)]
pub(crate) struct ConnectedEndpointList(HashMap<EndpointId, ConnectedEndpoint>);

impl ConnectedEndpointList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, epid: EndpointId, address: Address, timestamp: u64, data_sender: DataSender) -> bool {
        match self.0.entry(epid) {
            Entry::Occupied(_) => false,
            Entry::Vacant(entry) => {
                entry.insert(ConnectedEndpoint {
                    epid,
                    address,
                    timestamp,
                    data_sender,
                    duplicate: None,
                });
                true
            }
        }
    }

    pub fn contains(&self, epid: &EndpointId) -> bool {
        self.0.contains_key(epid)
    }

    pub fn remove(&mut self, epid: &EndpointId) -> bool {
        let duplicate = if let Some(endpoint) = self.0.remove(epid) {
            endpoint.duplicate
        } else {
            return false;
        };

        // NOTE: this should ideally not be necessary since duplicates are already removed
        if let Some(duplicate) = duplicate {
            self.0.remove(&duplicate);
        }

        true
    }

    pub fn set_duplicate(&mut self, epid: EndpointId, other: EndpointId) -> bool {
        if !self.0.get_mut(&epid).map_or(false, |endpoint| {
            if endpoint.duplicate.is_some() {
                panic!("Trying to replace an already set duplicate.");
            } else {
                endpoint.duplicate.replace(other).is_none()
            }
        }) {
            return false;
        }

        self.0.get_mut(&other).map_or(false, |other| {
            if other.duplicate.is_some() {
                panic!("Trying to replace an already set duplicate.");
            } else {
                other.duplicate.replace(epid).is_none()
            }
        })
    }

    pub fn remove_duplicate(&mut self, epid: &EndpointId, other: &EndpointId) -> bool {
        let (remove_epid, remain_epid) = if let Some(endpoint) = self.0.get(epid) {
            if let Some(other) = self.0.get(other) {
                // NOTE: no IP equality check (we let the protocol layer decide what's a duplicate)
                // NOTE: we drop the younger connection by conventions
                if other.timestamp >= endpoint.timestamp {
                    (other.epid, endpoint.epid)
                } else {
                    (endpoint.epid, other.epid)
                }
            } else {
                return false;
            }
        } else {
            return false;
        };

        if !self.0.remove(&remove_epid).is_some() {
            return false;
        }

        // NOTE: calling unwrap is safe, because we know already that 'remain_epid' exists.
        self.0.get_mut(&remain_epid).unwrap().duplicate = None;

        true
    }

    pub fn has_duplicate(&self, epid: &EndpointId) -> bool {
        self.0.get(epid).map_or(false, |endpoint| {
            endpoint
                .duplicate
                .map_or(false, |ref duplicate| self.0.get(duplicate).is_some())
        })
    }

    pub fn is_duplicate(&self, epid: &EndpointId) -> bool {
        self.0
            .iter()
            .any(|(_, v)| v.duplicate.map_or(false, |ref other| other == epid))
    }

    pub async fn send(&mut self, data: Vec<u8>, epid: &EndpointId) -> Result<bool, WorkerError> {
        if let Some(connected_endpoint) = self.0.get_mut(epid) {
            connected_endpoint.data_sender.send(data).await?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
