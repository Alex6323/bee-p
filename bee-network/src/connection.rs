use crate::endpoint::EndpointId;

use async_std::sync::Arc;
use futures::channel::mpsc;
use futures::sink::SinkExt;

use std::collections::hash_map::{
    Entry,
    Iter,
    IterMut,
};
use std::collections::HashMap;

pub type BytesSender = mpsc::Sender<Arc<Vec<u8>>>;
pub type BytesReceiver = mpsc::Receiver<Arc<Vec<u8>>>;

const MAX_BUFFER_SIZE: usize = 1654;
const BYTES_CHANNEL_CAPACITY: usize = 10000;

pub fn channel() -> (BytesSender, BytesReceiver) {
    mpsc::channel(BYTES_CHANNEL_CAPACITY)
}

pub struct ConnectionPool {
    inner: HashMap<EndpointId, BytesSender>,
}

impl ConnectionPool {
    pub fn new() -> Self {
        Self { inner: HashMap::new() }
    }

    pub fn size(&self) -> usize {
        self.inner.len()
    }

    pub fn insert(&mut self, id: EndpointId, sender: BytesSender) -> bool {
        match self.inner.entry(id.clone()) {
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

    pub fn sender(&mut self, id: &EndpointId) -> Option<&mut BytesSender> {
        self.inner.get_mut(id)
    }

    pub fn contains(&self, id: &EndpointId) -> bool {
        self.inner.contains_key(id)
    }

    pub async fn broadcast(&mut self, bytes: Vec<u8>) {
        let bytes = Arc::new(bytes);

        for (_, sender) in self.inner.iter_mut() {
            sender.send(Arc::clone(&bytes)).await;
        }
    }

    pub async fn send(&mut self, bytes: Vec<u8>, to: &EndpointId) {
        let bytes = Arc::new(bytes);

        if let Some(sender) = self.sender(&to) {
            sender.send(bytes).await;
        }
    }
}
