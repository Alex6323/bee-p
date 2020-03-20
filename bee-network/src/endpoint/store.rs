use super::{
    Endpoint,
    EndpointId,
};
use std::collections::hash_map::{
    Entry,
    Iter,
    IterMut,
};
use std::collections::HashMap;

pub struct Endpoints {
    inner: HashMap<EndpointId, Endpoint>,
}

impl Endpoints {
    pub fn new() -> Self {
        Self { inner: HashMap::new() }
    }

    pub fn num(&self) -> usize {
        self.inner.len()
    }

    pub fn insert(&mut self, ep: Endpoint) -> bool {
        match self.inner.entry(ep.id) {
            Entry::Occupied(_) => false,
            Entry::Vacant(entry) => {
                entry.insert(ep);
                true
            }
        }
    }

    pub fn remove(&mut self, id: &EndpointId) -> bool {
        self.inner.remove(id).is_some()
    }

    pub fn get(&self, id: &EndpointId) -> Option<&Endpoint> {
        self.inner.get(id)
    }

    pub fn get_mut(&mut self, id: &EndpointId) -> Option<&mut Endpoint> {
        self.inner.get_mut(id)
    }

    pub fn iter(&self) -> Iter<EndpointId, Endpoint> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<EndpointId, Endpoint> {
        self.inner.iter_mut()
    }

    pub fn contains(&self, id: &EndpointId) -> bool {
        self.inner.contains_key(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::url::{
        Protocol,
        Url,
    };
    use async_std::task::block_on;

    #[test]
    fn insert_and_remove_from_store() {
        let mut store = Endpoints::new();
        assert_eq!(0, store.num(), "Incorrect store size");

        let url = block_on(Url::from_str_with_port("udp://localhost:16000")).unwrap();
        let ep = Endpoint::from_url(url);
        let ep_id = ep.id;

        assert!(store.insert(ep), "Insertion failed");
        assert_eq!(1, store.num(), "Incorrect store size");

        assert!(store.remove(&ep_id));
        assert_eq!(0, store.num(), "Incorrect store size");
    }

    #[test]
    fn iterate_store() {
        let mut store = Endpoints::new();

        let url1 = block_on(Url::from_str_with_port("udp://localhost:16000")).unwrap();
        let ep1 = Endpoint::from_url(url1);

        let url2 = block_on(Url::from_str_with_port("udp://localhost:17000")).unwrap();
        let ep2 = Endpoint::from_url(url2);

        assert!(store.insert(ep1), "Insertion failed");
        assert!(store.insert(ep2), "Insertion failed");
        assert_eq!(2, store.num(), "Incorrect store size");

        let mut count = 0;
        for (_id, _ep) in store.iter() {
            count += 1;
        }
        assert_eq!(2, count, "Immutable iteration failed");

        let mut count = 0;
        for (_id, _ep) in store.iter_mut() {
            count += 1;
        }
        assert_eq!(2, count, "Mutable iteration failed");
    }

    #[test]
    fn get_endpoint_from_store() {
        let mut store = Endpoints::new();

        let url1 = block_on(Url::from_str_with_port("udp://localhost:16000")).unwrap();
        let ep1 = Endpoint::from_url(url1);
        let ep1_id = ep1.id;

        store.insert(ep1);

        assert!(store.get(&ep1_id).is_some(), "Getting endpoint from store failed");
    }

    #[test]
    fn get_mutable_endpoint_from_store() {
        let mut store = Endpoints::new();

        let url1 = block_on(Url::from_str_with_port("udp://localhost:16000")).unwrap();
        let ep1 = Endpoint::from_url(url1);
        let ep1_id = ep1.id;

        store.insert(ep1);

        assert!(store.get_mut(&ep1_id).is_some(), "Getting endpoint from store failed");

        if let Some(ep1) = store.get_mut(&ep1_id) {
            assert_eq!(Protocol::Udp, ep1.protocol);
            ep1.protocol = Protocol::Tcp;
            assert_eq!(Protocol::Tcp, ep1.protocol);
        } else {
            assert!(false, "Getting mutable endpoint from store failed");
        }
    }
}
