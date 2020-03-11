use crate::endpoint::EndpointId;

use futures::channel::mpsc;
use futures::sink::SinkExt;

use std::collections::HashMap;

pub type BytesSender = mpsc::Sender<Vec<u8>>;
pub type BytesReceiver = mpsc::Receiver<Vec<u8>>;

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

    pub fn insert(&mut self, ep_id: EndpointId, sender: BytesSender) {
        self.inner.insert(ep_id, sender);
    }

    pub fn remove(&mut self, ep_id: &EndpointId) -> bool {
        self.inner.remove(ep_id).is_some()
    }

    pub fn get_mut(&mut self, ep_id: &EndpointId) -> Option<&mut BytesSender> {
        self.inner.get_mut(ep_id)
    }

    pub async fn broadcast(&mut self, bytes: Vec<u8>) {
        for (_, sender) in self.inner.iter_mut() {
            // TODO: do not clone (use Arc)
            sender.send(bytes.clone()).await;
        }
    }

    pub async fn send(&mut self, bytes: Vec<u8>, to: &EndpointId) {
        if let Some(sender) = self.get_mut(&to) {
            sender.send(bytes).await;
        }
    }
}
