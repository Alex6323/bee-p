// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::{Endpoint, EndpointId};
use std::collections::{
    hash_map::{Entry, Iter, IterMut},
    HashMap,
};

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

    // TODO: see if we need this API in the future.
    #[allow(dead_code)]
    pub fn get(&self, id: &EndpointId) -> Option<&Endpoint> {
        self.inner.get(id)
    }

    pub fn get_mut(&mut self, id: &EndpointId) -> Option<&mut Endpoint> {
        self.inner.get_mut(id)
    }

    // TODO: see if we need this API in the future.
    #[allow(dead_code)]
    pub fn iter(&self) -> Iter<EndpointId, Endpoint> {
        self.inner.iter()
    }

    // TODO: see if we need this API in the future.
    #[allow(dead_code)]
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
    use crate::address::url::{Protocol, Url};
    use async_std::task::block_on;

    #[test]
    fn insert_and_remove_from_store() {
        let mut store = Endpoints::new();
        assert_eq!(0, store.num(), "Incorrect store size");

        let url = block_on(Url::from_url_str("udp://localhost:16000")).unwrap();
        let ep = Endpoint::from_url(url);
        let epid = ep.id;

        assert!(store.insert(ep), "Insertion failed");
        assert_eq!(1, store.num(), "Incorrect store size");

        assert!(store.remove(&epid));
        assert_eq!(0, store.num(), "Incorrect store size");
    }

    #[test]
    fn iterate_store() {
        let mut store = Endpoints::new();

        let url1 = block_on(Url::from_url_str("udp://localhost:16000")).unwrap();
        let ep1 = Endpoint::from_url(url1);

        let url2 = block_on(Url::from_url_str("udp://localhost:17000")).unwrap();
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

        let url1 = block_on(Url::from_url_str("udp://localhost:16000")).unwrap();
        let ep1 = Endpoint::from_url(url1);
        let epid1 = ep1.id;

        store.insert(ep1);

        assert!(store.get(&epid1).is_some(), "Getting endpoint from store failed");
    }

    #[test]
    fn get_mutable_endpoint_from_store() {
        let mut store = Endpoints::new();

        let url1 = block_on(Url::from_url_str("udp://localhost:16000")).unwrap();
        let ep1 = Endpoint::from_url(url1);
        let epid1 = ep1.id;

        store.insert(ep1);

        assert!(store.get_mut(&epid1).is_some(), "Getting endpoint from store failed");

        if let Some(ep1) = store.get_mut(&epid1) {
            assert_eq!(Protocol::Udp, ep1.protocol);
            ep1.protocol = Protocol::Tcp;
            assert_eq!(Protocol::Tcp, ep1.protocol);
        } else {
            panic!("Getting mutable endpoint from store failed");
        }
    }
}
