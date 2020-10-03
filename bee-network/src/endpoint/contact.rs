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

use crate::util::{self, Port, TransportProtocol};

use super::{EndpointId, Error};

use dashmap::{mapref::entry::Entry, DashMap};
use url;

use std::{
    fmt,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndpointContactParams {
    pub domain_name_or_ip_address: String,
    pub port: Port,
    pub transport_protocol: TransportProtocol,
    last_socket_address: SocketAddr,
}

impl EndpointContactParams {
    #[allow(dead_code)]
    pub fn from_socket_address(socket_address: SocketAddr, transport_protocol: TransportProtocol) -> Self {
        Self {
            domain_name_or_ip_address: socket_address.ip().to_string(),
            port: socket_address.port(),
            transport_protocol,
            last_socket_address: socket_address,
        }
    }

    pub async fn from_url(url: &str) -> Result<Self, Error> {
        if let Ok(url) = url::Url::parse(url) {
            let domain_name_or_ip_address = url.host_str().ok_or(Error::UrlParseFailure)?.to_string();
            let port = url.port().ok_or(Error::UrlParseFailure)?;
            let transport_protocol = match url.scheme() {
                "tcp" => TransportProtocol::Tcp,
                "udp" => TransportProtocol::Udp,
                "" => return Err(Error::UnspecifiedTransportProtocol),
                _ => return Err(Error::UnsupportedTransportProtocol),
            };

            let socket_address = &format!("{}:{}", domain_name_or_ip_address, port)[..];
            let last_socket_address = util::resolve_address(socket_address)
                .await
                .map_err(|_| Error::DnsFailure)?
                .into();

            Ok(Self {
                domain_name_or_ip_address,
                port,
                transport_protocol,
                last_socket_address,
            })
        } else {
            Err(Error::UrlParseFailure)
        }
    }

    pub async fn socket_address(&mut self, refresh: bool) -> Result<SocketAddr, Error> {
        if refresh {
            let socket_address = &format!("{}:{}", self.domain_name_or_ip_address, self.port)[..];
            let socket_address = util::resolve_address(socket_address)
                .await
                .map_err(|_| Error::DnsFailure)?
                .into();

            self.last_socket_address = socket_address;
        }

        Ok(self.last_socket_address)
    }

    pub fn create_epid(&self) -> EndpointId {
        EndpointId::new(self.transport_protocol, self.last_socket_address)
    }
}

impl fmt::Display for EndpointContactParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}://{}:{}",
            self.transport_protocol, self.domain_name_or_ip_address, self.port
        )
    }
}

const DEFAULT_CONTACTLIST_CAPACITY: usize = 16;

#[derive(Clone, Debug)]
pub struct EndpointContactList(Arc<DashMap<EndpointId, EndpointContactParams>>);

impl EndpointContactList {
    pub fn new() -> Self {
        Self(Arc::new(DashMap::with_capacity(DEFAULT_CONTACTLIST_CAPACITY)))
    }

    pub fn insert(&self, epid: EndpointId, endpoint_params: EndpointContactParams) -> bool {
        match self.0.entry(epid) {
            Entry::Occupied(_) => false,
            Entry::Vacant(entry) => {
                entry.insert(endpoint_params);
                true
            }
        }
    }

    pub fn contains(&self, epid: EndpointId) -> bool {
        self.0.contains_key(&epid)
    }

    pub fn contains_ip_address(&self, ip_address: IpAddr, _refresh: bool) -> bool {
        // TODO: 'refresh' IPs
        self.0
            .iter()
            .any(|entry| entry.value().last_socket_address.ip() == ip_address)
    }

    pub fn remove(&self, epid: EndpointId) -> bool {
        self.0.remove(&epid).is_some()
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    // TODO: impl refresh
    pub fn get(&self, epid: EndpointId) -> Option<EndpointContactParams> {
        // NOTE: no async closures :(
        self.0.get(&epid).map(|entry| entry.value().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_ipv4_tcp_endpoint_params_from_url() {
        let mut endpoint_params = EndpointContactParams::from_url("tcp://127.0.0.1:15600").await.unwrap();
        let endpoint_address = endpoint_params.socket_address(false).await.unwrap();

        assert!(endpoint_address.is_ipv4());
        assert_eq!(TransportProtocol::Tcp, endpoint_params.transport_protocol);
        assert_eq!("tcp://127.0.0.1:15600", endpoint_params.to_string());
    }

    #[tokio::test]
    async fn create_ipv4_udp_endpoint_params_from_url() {
        let mut endpoint_params = EndpointContactParams::from_url("udp://127.0.0.1:15600").await.unwrap();
        let endpoint_address = endpoint_params.socket_address(false).await.unwrap();

        assert!(endpoint_address.is_ipv4());
        assert_eq!(TransportProtocol::Udp, endpoint_params.transport_protocol);
        assert_eq!("udp://127.0.0.1:15600", endpoint_params.to_string());
    }

    #[tokio::test]
    async fn create_ipv6_tcp_endpoint_params_from_url() {
        let mut endpoint_params = EndpointContactParams::from_url("tcp://[::1]:15600").await.unwrap();
        let endpoint_address = endpoint_params.socket_address(false).await.unwrap();

        assert!(endpoint_address.is_ipv6());
        assert_eq!(TransportProtocol::Tcp, endpoint_params.transport_protocol);
        assert_eq!("tcp://[::1]:15600", endpoint_params.to_string());
    }

    #[tokio::test]
    async fn create_ipv6_udp_endpoint_params_from_url() {
        let mut endpoint_params = EndpointContactParams::from_url("udp://[::1]:15600").await.unwrap();
        let endpoint_address = endpoint_params.socket_address(false).await.unwrap();

        assert!(endpoint_address.is_ipv6());
        assert_eq!(TransportProtocol::Udp, endpoint_params.transport_protocol);
        assert_eq!("udp://[::1]:15600", endpoint_params.to_string());
    }
}

// #[cfg(test)]
// mod tests2 {
//     use super::*;

//     use std::collections::hash_map::{Iter, IterMut};
//     use std::net::SocketAddr;

//     impl EndpointList {
//         pub fn iter(&self) -> Iter<EndpointId, Endpoint> {
//             self.0.iter()
//         }

//         pub fn iter_mut(&mut self) -> IterMut<EndpointId, Endpoint> {
//             self.0.iter_mut()
//         }

//         pub fn get(&self, epid: &EndpointId) -> Option<&Endpoint> {
//             self.0.get(epid)
//         }
//     }

//     #[tokio::test]
//     async fn insert_and_remove_from_endpoint_list() {
//         let mut endpoints = EndpointList::new();
//         assert_eq!(0, endpoints.len(), "Incorrect endpoint list size");

//         let url = Url::from_str("udp://localhost:16000").await.unwrap();
//         let endpoint = Endpoint::from_url(url);
//         let epid = endpoint.epid;

//         assert!(endpoints.insert(endpoint), "Insertion failed");
//         assert_eq!(1, endpoints.len(), "Incorrect endpoint list size");

//         assert!(endpoints.remove(&epid));
//         assert_eq!(0, endpoints.len(), "Incorrect endpoint list size");
//     }

//     #[test]
//     fn iterate_endpoint_list() {
//         let mut endpoints = EndpointList::new();

//         let url1 = Url::from_str("udp://localhost:16000").unwrap();
//         let ep1 = Endpoint::from_url(url1);

//         let url2 = Url::from_str("udp://localhost:17000").unwrap();
//         let ep2 = Endpoint::from_url(url2);

//         assert!(endpoints.insert(ep1), "Insertion failed");
//         assert!(endpoints.insert(ep2), "Insertion failed");
//         assert_eq!(2, endpoints.len(), "Incorrect endpoint list size");

//         let mut count = 0;
//         for (_id, _ep) in endpoints.iter() {
//             count += 1;
//         }
//         assert_eq!(2, count, "Immutable iteration failed");

//         let mut count = 0;
//         for (_id, _ep) in endpoints.iter_mut() {
//             count += 1;
//         }
//         assert_eq!(2, count, "Mutable iteration failed");
//     }

//     #[test]
//     fn get_endpoint_from_endpoint_list() {
//         let mut endpoints = EndpointList::new();

//         let url1 = Url::from_str("udp://localhost:16000").unwrap();
//         let ep1 = Endpoint::from_url(url1);
//         let epid1 = ep1.epid;

//         endpoints.insert(ep1);

//         assert!(
//             endpoints.get(&epid1).is_some(),
//             "Getting endpoint from endpoint list failed"
//         );
//     }

//     #[test]
//     fn get_mutable_endpoint_from_endpoint_list() {
//         let mut endpoints = EndpointList::new();

//         let url1 = Url::from_str("udp://localhost:16000").unwrap();
//         let ep1 = Endpoint::from_url(url1);
//         let epid1 = ep1.epid;

//         endpoints.insert(ep1);

//         assert!(
//             endpoints.get_mut(&epid1).is_some(),
//             "Getting endpoint from endpoint list failed"
//         );

//         if let Some(ep1) = endpoints.get_mut(&epid1) {
//             assert_eq!(TransportProtocol::Udp, ep1.protocol);
//             ep1.protocol = TransportProtocol::Tcp;
//             assert_eq!(TransportProtocol::Tcp, ep1.protocol);
//         } else {
//             panic!("Getting mutable endpoint from endpoint list failed");
//         }
//     }

//     #[test]
//     fn create_epid_from_address() {
//         let address = "127.0.0.1:16000".parse::<SocketAddr>.unwrap();
//         let epid: EndpointId = address.into();

//         assert_eq!("127.0.0.1:16000", epid.to_string());
//     }

//     #[test]
//     fn create_epid_from_url() {
//         let url = Url::from_str("tcp://[::1]:16000").unwrap();
//         let epid: EndpointId = url.into();

//         assert_eq!("[::1]:16000", epid.to_string());
//     }

//     #[test]
//     fn create_endpoint_from_url() {
//         let url = Url::from_str("udp://[::1]:16000").unwrap();
//         let endpoint = Endpoint::from_url(url);

//         assert_eq!("[::1]:16000", endpoint.epid.to_string());
//         assert_eq!(Protocol::Udp, endpoint.protocol);
//         assert_eq!("[::1]:16000", endpoint.address.to_string());
//     }
// }
