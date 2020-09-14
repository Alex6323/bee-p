use crate::util::net;

use super::{Error, Port, TransportProtocol};

use url;

use std::{
    collections::{hash_map::Entry, HashMap},
    fmt,
    net::SocketAddr,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndpointContactParams {
    pub domain_name_or_ip_address: String,
    pub port: Port,
    pub transport_protocol: TransportProtocol,
    dns_lookup_cache: Option<SocketAddr>,
}

impl EndpointContactParams {
    pub fn from_socket_address(socket_address: SocketAddr, transport_protocol: TransportProtocol) -> Self {
        Self {
            domain_name_or_ip_address: socket_address.ip().to_string(),
            port: socket_address.port(),
            transport_protocol,
            dns_lookup_cache: Some(socket_address),
        }
    }

    pub fn from_url(url: &str) -> Result<Self, Error> {
        if let Ok(url) = url::Url::parse(url) {
            let domain_name_or_ip_address = url.host_str().ok_or(Error::UrlParseFailure)?.to_string();
            let port = url.port().ok_or(Error::UrlParseFailure)?;
            let transport_protocol = match url.scheme() {
                "tcp" => TransportProtocol::Tcp,
                "udp" => TransportProtocol::Udp,
                "" => return Err(Error::UnspecifiedTransportProtocol),
                _ => return Err(Error::UnsupportedTransportProtocol),
            };

            Ok(Self {
                domain_name_or_ip_address,
                port,
                transport_protocol,
                dns_lookup_cache: None,
            })
        } else {
            Err(Error::UrlParseFailure)
        }
    }

    pub async fn socket_address(&mut self, refresh: bool) -> Result<SocketAddr, Error> {
        if refresh || self.dns_lookup_cache.is_none() {
            let address = &format!("{}:{}", self.domain_name_or_ip_address, self.port)[..];
            let socket_address = net::resolve_address(address)
                .await
                .map_err(|_| Error::DnsFailure)?
                .into();

            self.dns_lookup_cache = Some(socket_address);
        }

        Ok(*self.dns_lookup_cache.as_ref().unwrap())
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

#[derive(Default)]
pub struct EndpointContactList(HashMap<String, EndpointContactParams>);

impl EndpointContactList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, url: String, params: EndpointContactParams) -> bool {
        match self.0.entry(url) {
            Entry::Occupied(_) => false,
            Entry::Vacant(entry) => {
                entry.insert(params);
                true
            }
        }
    }

    #[allow(dead_code)]
    pub fn contains(&self, url: &str) -> bool {
        self.0.contains_key(url)
    }
    pub fn remove(&mut self, url: &str) -> bool {
        self.0.remove(url).is_some()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[allow(dead_code)]
    pub fn get(&self, url: &str) -> Option<&EndpointContactParams> {
        self.0.get(url)
    }

    pub fn get_mut(&mut self, url: &str) -> Option<&mut EndpointContactParams> {
        self.0.get_mut(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_tcp_connection_info_from_address_and_protocol() {
        let socket_address = EndpointContactParams::from_url("tcp://127.0.0.1:15600")
            .unwrap()
            .socket_address(true)
            .await
            .unwrap();

        let params = EndpointContactParams::from_socket_address(socket_address, TransportProtocol::Tcp);

        assert_eq!(TransportProtocol::Tcp, params.transport_protocol);
        assert_eq!("tcp://127.0.0.1:15600", params.to_string());
    }

    #[tokio::test]
    async fn create_udp_connection_info_from_address_and_protocol() {
        let socket_address = EndpointContactParams::from_url("udp://127.0.0.1:15600")
            .unwrap()
            .socket_address(true)
            .await
            .unwrap();

        let params = EndpointContactParams::from_socket_address(socket_address, TransportProtocol::Udp);

        assert_eq!(TransportProtocol::Udp, params.transport_protocol);
        assert_eq!("udp://127.0.0.1:15600", params.to_string());
    }

    #[test]
    fn create_tcp_connection_info_from_url() {
        let params = EndpointContactParams::from_url("tcp://127.0.0.1:15600").expect("parsing url failed");

        assert_eq!(TransportProtocol::Tcp, params.transport_protocol);
        assert_eq!("tcp://127.0.0.1:15600", params.to_string());
    }

    #[test]
    fn create_udp_connection_info_from_url() {
        let params = EndpointContactParams::from_url("udp://127.0.0.1:15600").expect("parsing url failed");

        assert_eq!(TransportProtocol::Udp, params.transport_protocol);
        assert_eq!("udp://127.0.0.1:15600", params.to_string());
    }

    #[tokio::test]
    async fn create_ipv6_connection_info_from_url() {
        let params = EndpointContactParams::from_url("tcp://[::1]:15600").expect("parsing url failed");

        let socket_address = params.socket_address(false).await.unwrap();

        assert!(socket_address.is_ipv6());
        assert_eq!(TransportProtocol::Tcp, params.transport_protocol);
        assert_eq!("tcp://[::1]:15600", params.to_string());
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
