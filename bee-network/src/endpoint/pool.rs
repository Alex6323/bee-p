use super::{Endpoint, EndpointId};

use std::collections::HashMap;
use std::collections::hash_map::{Entry, Iter, IterMut};

pub struct EndpointPool {
    inner: HashMap<EndpointId, Endpoint>,
}

impl EndpointPool {

    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn size(&self) -> usize {
        self.inner.len()
    }

    // TODO: instead of ignoring the insertion attempt, if a endpoint with the same ID already
    // exists, should we replace?
    pub fn insert(&mut self, ep: Endpoint) -> bool {
        match self.inner.entry(ep.id.clone()) {
            Entry::Occupied(_) => false,
            Entry::Vacant(entry) => {
                entry.insert(ep);
                true
            }
        }
    }

    pub fn remove(&mut self, ep_id: &EndpointId) -> bool {
        self.inner.remove(ep_id).is_some()
    }

    pub fn get(&self, ep_id: &EndpointId) -> Option<&Endpoint> {
        self.inner.get(ep_id)
    }

    pub fn get_mut(&mut self, ep_id: &EndpointId) -> Option<&mut Endpoint> {
        self.inner.get_mut(ep_id)
    }

    pub fn iter(&self) -> Iter<EndpointId, Endpoint> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<EndpointId, Endpoint> {
        self.inner.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::EndpointState;
    use crate::address::url::Url;
    use async_std::task::block_on;

    #[test]
    fn insert_and_remove_from_pool() {
        let mut pool = EndpointPool::new();
        assert_eq!(0, pool.size(), "Incorrect pool size");

        let url = block_on(Url::from_str_with_port("udp://localhost:16000")).unwrap();
        let ep = Endpoint::from_url(url);
        let ep_id = ep.id.clone();

        assert!(pool.insert(ep), "Insertion failed");
        assert_eq!(1, pool.size(), "Incorrect pool size");

        assert!(pool.remove(&ep_id));
        assert_eq!(0, pool.size(), "Incorrect pool size");
    }

    #[test]
    fn iterate_pool() {
        let mut pool = EndpointPool::new();

        let url1 = block_on(Url::from_str_with_port("udp://localhost:16000")).unwrap();
        let ep1 = Endpoint::from_url(url1);

        let url2 = block_on(Url::from_str_with_port("udp://localhost:17000")).unwrap();
        let ep2 = Endpoint::from_url(url2);

        assert!(pool.insert(ep1), "Insertion failed");
        assert!(pool.insert(ep2), "Insertion failed");
        assert_eq!(2, pool.size(), "Incorrect pool size");

        let mut count = 0;
        for (_id, _ep) in pool.iter() {
            count += 1;
        }
        assert_eq!(2, count, "Immutable iteration failed");

        let mut count = 0;
        for (_id, _ep) in pool.iter_mut() {
            count += 1;
        }
        assert_eq!(2, count, "Mutable iteration failed");
    }

    #[test]
    fn get_endpoint_from_pool() {
        let mut pool = EndpointPool::new();

        let url1 = block_on(Url::from_str_with_port("udp://localhost:16000")).unwrap();
        let ep1 = Endpoint::from_url(url1);
        let ep1_id = ep1.id.clone();

        pool.insert(ep1);

        assert!(pool.get(&ep1_id).is_some(), "Getting endpoint from pool failed");
    }

    #[test]
    fn get_mutable_endpoint_from_pool() {
        let mut pool = EndpointPool::new();

        let url1 = block_on(Url::from_str_with_port("udp://localhost:16000")).unwrap();
        let ep1 = Endpoint::from_url(url1);
        let ep1_id = ep1.id.clone();

        pool.insert(ep1);

        assert!(pool.get_mut(&ep1_id).is_some(), "Getting endpoint from pool failed");

        if let Some(ep1) = pool.get_mut(&ep1_id) {
            assert_eq!(EndpointState::Disconnected, ep1.state);
            ep1.state = EndpointState::Connected;
            assert_eq!(EndpointState::Connected, ep1.state);
        } else {
            assert!(false, "Getting mutable endpoint from pool failed");
        }
    }
}