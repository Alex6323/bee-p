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

use crate::{constants::BYTES_CHANNEL_CAPACITY, endpoint::EndpointId as EpId};

use bee_common::shutdown::Result;

use async_std::sync::Arc;
use futures::{channel::mpsc, sink::SinkExt};

use std::collections::{hash_map::Entry, HashMap};

// TODO: rename to `MessageSender`, `MessageReceiver`.
pub type BytesSender = mpsc::Sender<Arc<Vec<u8>>>;
pub type BytesReceiver = mpsc::Receiver<Arc<Vec<u8>>>;

// TODO: rename to `message_channel`
pub fn bytes_channel() -> (BytesSender, BytesReceiver) {
    mpsc::channel(BYTES_CHANNEL_CAPACITY)
}

/// Responsible for sending messages (i.e. chunks of bytes) to the writer tasks handling the recipients.
pub struct Outbox {
    inner: HashMap<EpId, BytesSender>,
}

impl Outbox {
    /// Creates a new instance of `Self`.
    pub fn new() -> Self {
        Self { inner: HashMap::new() }
    }

    /// Inserts a new outgoing communication channel to a recipient referred to by its `EndpointId`.
    ///
    /// NOTE: Instead of replacing, this method inserts only, if there is no entry with that endpoint id yet.
    pub fn insert(&mut self, epid: EpId, sender: BytesSender) -> bool {
        match self.inner.entry(epid) {
            Entry::Occupied(_) => false,
            Entry::Vacant(entry) => {
                entry.insert(sender);
                true
            }
        }
    }

    /// Removes an outgoing communication channel.
    ///
    /// NOTE: Releasing the channel sender half will cause the writer task to end.
    pub fn remove(&mut self, id: &EpId) -> bool {
        self.inner.remove(id).is_some()
    }

    /// Sends `bytes` to `receiver`.
    ///
    /// Returns `true` if the send was successful.
    pub async fn send(&mut self, bytes: Vec<u8>, recipient: &EpId) -> Result<bool> {
        let bytes = Arc::new(bytes);
        if let Some(sender) = self.inner.get_mut(recipient) {
            sender.send(bytes).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Multicasts `bytes` to the `receivers`.
    ///
    /// NOTE: The multicast is considered to be successful, if at least
    /// one send is successful.
    pub async fn multicast(&mut self, bytes: Vec<u8>, recipients: &[EpId]) -> Result<bool> {
        let bytes = Arc::new(bytes);
        let mut num_sends = 0;

        // TODO: Do not block!
        for (epid, sender) in self.inner.iter_mut() {
            if recipients.contains(epid) {
                sender.send(Arc::clone(&bytes)).await?;
                num_sends += 1;
            }
        }

        Ok(num_sends > 0)
    }

    /// Broadcasts `bytes` using all available connections from the pool.
    ///
    /// NOTE: The broadcast is considered to be successful, if at least
    /// one send is successful.
    pub async fn broadcast(&mut self, bytes: Vec<u8>) -> Result<bool> {
        let bytes = Arc::new(bytes);
        let mut num_sends = 0;

        // TODO: Do not block!
        for (_, sender) in self.inner.iter_mut() {
            sender.send(Arc::clone(&bytes)).await?;
            num_sends += 1;
        }

        Ok(num_sends > 0)
    }
}
