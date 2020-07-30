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

use crate::endpoint::EndpointId;

use bee_common::worker::Error as WorkerError;

use async_std::sync::Arc;
use futures::{channel::mpsc, sink::SinkExt};

use std::collections::{hash_map::Entry, HashMap};

const BYTES_CHANNEL_CAPACITY: usize = 10000;

pub type BytesSender = mpsc::Sender<Arc<Vec<u8>>>;
pub type BytesReceiver = mpsc::Receiver<Arc<Vec<u8>>>;

pub fn bytes_channel() -> (BytesSender, BytesReceiver) {
    mpsc::channel(BYTES_CHANNEL_CAPACITY)
}

pub struct Outbox {
    inner: HashMap<EndpointId, BytesSender>,
}

impl Outbox {
    pub fn new() -> Self {
        Self { inner: HashMap::new() }
    }

    pub fn insert(&mut self, epid: EndpointId, sender: BytesSender) -> bool {
        match self.inner.entry(epid) {
            Entry::Occupied(_) => false,
            Entry::Vacant(entry) => {
                entry.insert(sender);
                true
            }
        }
    }

    pub fn remove(&mut self, id: &EndpointId) -> bool {
        self.inner.remove(id).is_some()
    }

    pub async fn send(&mut self, bytes: Vec<u8>, epid: &EndpointId) -> Result<bool, WorkerError> {
        let bytes = Arc::new(bytes);

        if let Some(sender) = self.inner.get_mut(epid) {
            sender.send(bytes).await?;

            Ok(true)
        } else {
            Ok(false)
        }
    }
}
