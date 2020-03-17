use crate::endpoint::EndpointId as EpId;
use crate::errors::ActorResult as R;

use async_std::sync::Arc;
use futures::channel::mpsc;
use futures::sink::SinkExt;

use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub type BytesSender = mpsc::Sender<Arc<Vec<u8>>>;
pub type BytesReceiver = mpsc::Receiver<Arc<Vec<u8>>>;

const MAX_BUFFER_SIZE: usize = 1654;
const BYTES_CHANNEL_CAPACITY: usize = 10000;

pub fn bytes_channel() -> (BytesSender, BytesReceiver) {
    mpsc::channel(BYTES_CHANNEL_CAPACITY)
}

pub struct ConnectionPool {
    inner: HashMap<EpId, BytesSender>,
}

impl ConnectionPool {
    /// Creates a new `ConnectionPool`.
    pub fn new() -> Self {
        Self { inner: HashMap::new() }
    }

    /// Returns the size of the pool.
    pub fn size(&self) -> usize {
        self.inner.len()
    }

    /// Inserts a `sender` to the pool.
    pub fn insert(&mut self, id: EpId, sender: BytesSender) -> bool {
        match self.inner.entry(id.clone()) {
            Entry::Occupied(_) => false,
            Entry::Vacant(entry) => {
                entry.insert(sender);
                true
            }
        }
    }

    /// Removes a `sender` associated with an endpoint.
    pub fn remove(&mut self, id: &EpId) -> bool {
        self.inner.remove(id).is_some()
    }

    /// Checks whether the specified endpoint belongs to the pool.
    pub fn contains(&self, id: &EpId) -> bool {
        self.inner.contains_key(id)
    }

    /// Sends `bytes` to `receiver`.
    ///
    /// Returns `true` if the send was successful.
    pub async fn send(&mut self, bytes: Arc<Vec<u8>>, receiver: &EpId) -> R<bool> {
        if let Some(sender) = self.inner.get_mut(receiver) {
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
    pub async fn multicast(&mut self, bytes: Arc<Vec<u8>>, receivers: &Vec<EpId>) -> R<bool> {
        let mut num_sends = 0;

        for (epid, sender) in self.inner.iter_mut() {
            if receivers.contains(epid) {
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
    pub async fn broadcast(&mut self, bytes: Arc<Vec<u8>>) -> R<bool> {
        let mut num_sends = 0;

        for (_, sender) in self.inner.iter_mut() {
            sender.send(Arc::clone(&bytes)).await?;
            num_sends += 1;
        }

        Ok(num_sends > 0)
    }
}
