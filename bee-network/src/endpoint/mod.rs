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

pub mod allowlist;
pub mod connected;
pub mod worker;

use crate::address::{
    url::{Protocol, Url},
    Address,
};

use std::{
    collections::{hash_map::Entry, HashMap},
    fmt,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EndpointId(Address);

impl From<Address> for EndpointId {
    fn from(addr: Address) -> Self {
        Self(addr)
    }
}

// FIXME: async call in non-async trait
// impl From<Url> for EndpointId {
//     fn from(mut url: Url) -> Self {
//         Self(url.address(true).unwrap().into())
//     }
// }

impl fmt::Display for EndpointId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct Endpoint {
    pub epid: EndpointId,
    pub address: Address,
    pub protocol: Protocol,
}

impl Endpoint {
    pub fn new(address: Address, protocol: Protocol) -> Self {
        Self {
            epid: address.into(),
            address,
            protocol,
        }
    }

    pub async fn from_url(mut url: Url) -> Self {
        // FIXME: unwrap
        let address = url.address(true).await.unwrap();
        let protocol = url.protocol;

        Endpoint::new(address, protocol)
    }
}

impl Eq for Endpoint {}
impl PartialEq for Endpoint {
    fn eq(&self, other: &Self) -> bool {
        self.epid == other.epid
    }
}

#[derive(Default)]
pub struct EndpointList(HashMap<EndpointId, Endpoint>);

impl EndpointList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, endpoint: Endpoint) -> bool {
        match self.0.entry(endpoint.epid) {
            Entry::Occupied(_) => false,
            Entry::Vacant(entry) => {
                entry.insert(endpoint);
                true
            }
        }
    }

    #[allow(dead_code)]
    pub fn contains(&self, epid: &EndpointId) -> bool {
        self.0.contains_key(epid)
    }
    pub fn remove(&mut self, epid: &EndpointId) -> bool {
        self.0.remove(epid).is_some()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get_mut(&mut self, epid: &EndpointId) -> Option<&mut Endpoint> {
        self.0.get_mut(epid)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::address::url::{Protocol, Url};
    use std::collections::hash_map::{Iter, IterMut};

    impl EndpointList {
        pub fn iter(&self) -> Iter<EndpointId, Endpoint> {
            self.0.iter()
        }

        pub fn iter_mut(&mut self) -> IterMut<EndpointId, Endpoint> {
            self.0.iter_mut()
        }

        pub fn get(&self, epid: &EndpointId) -> Option<&Endpoint> {
            self.0.get(epid)
        }
    }

    #[test]
    fn insert_and_remove_from_endpoint_list() {
        let mut endpoints = EndpointList::new();
        assert_eq!(0, endpoints.len(), "Incorrect endpoint list size");

        let url = Url::from_str("udp://localhost:16000").unwrap();
        let endpoint = Endpoint::from_url(url);
        let epid = endpoint.epid;

        assert!(endpoints.insert(endpoint), "Insertion failed");
        assert_eq!(1, endpoints.len(), "Incorrect endpoint list size");

        assert!(endpoints.remove(&epid));
        assert_eq!(0, endpoints.len(), "Incorrect endpoint list size");
    }

    #[test]
    fn iterate_endpoint_list() {
        let mut endpoints = EndpointList::new();

        let url1 = Url::from_str("udp://localhost:16000").unwrap();
        let ep1 = Endpoint::from_url(url1);

        let url2 = Url::from_str("udp://localhost:17000").unwrap();
        let ep2 = Endpoint::from_url(url2);

        assert!(endpoints.insert(ep1), "Insertion failed");
        assert!(endpoints.insert(ep2), "Insertion failed");
        assert_eq!(2, endpoints.len(), "Incorrect endpoint list size");

        let mut count = 0;
        for (_id, _ep) in endpoints.iter() {
            count += 1;
        }
        assert_eq!(2, count, "Immutable iteration failed");

        let mut count = 0;
        for (_id, _ep) in endpoints.iter_mut() {
            count += 1;
        }
        assert_eq!(2, count, "Mutable iteration failed");
    }

    #[test]
    fn get_endpoint_from_endpoint_list() {
        let mut endpoints = EndpointList::new();

        let url1 = Url::from_str("udp://localhost:16000").unwrap();
        let ep1 = Endpoint::from_url(url1);
        let epid1 = ep1.epid;

        endpoints.insert(ep1);

        assert!(
            endpoints.get(&epid1).is_some(),
            "Getting endpoint from endpoint list failed"
        );
    }

    #[test]
    fn get_mutable_endpoint_from_endpoint_list() {
        let mut endpoints = EndpointList::new();

        let url1 = Url::from_str("udp://localhost:16000").unwrap();
        let ep1 = Endpoint::from_url(url1);
        let epid1 = ep1.epid;

        endpoints.insert(ep1);

        assert!(
            endpoints.get_mut(&epid1).is_some(),
            "Getting endpoint from endpoint list failed"
        );

        if let Some(ep1) = endpoints.get_mut(&epid1) {
            assert_eq!(Protocol::Udp, ep1.protocol);
            ep1.protocol = Protocol::Tcp;
            assert_eq!(Protocol::Tcp, ep1.protocol);
        } else {
            panic!("Getting mutable endpoint from endpoint list failed");
        }
    }

    #[test]
    fn create_epid_from_address() {
        let address = Address::from_str("127.0.0.1:16000").unwrap();
        let epid: EndpointId = address.into();

        assert_eq!("127.0.0.1:16000", epid.to_string());
    }

    #[test]
    fn create_epid_from_url() {
        let url = Url::from_str("tcp://[::1]:16000").unwrap();
        let epid: EndpointId = url.into();

        assert_eq!("[::1]:16000", epid.to_string());
    }

    #[test]
    fn create_endpoint_from_url() {
        let url = Url::from_str("udp://[::1]:16000").unwrap();
        let endpoint = Endpoint::from_url(url);

        assert_eq!("[::1]:16000", endpoint.epid.to_string());
        assert_eq!(Protocol::Udp, endpoint.protocol);
        assert_eq!("[::1]:16000", endpoint.address.to_string());
    }
}
