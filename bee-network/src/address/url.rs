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

use super::{Address, Error, Port};
use crate::utils::net;

use url;

use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Protocol {
    Tcp,
    Udp,
}

impl Protocol {
    pub fn is_tcp(self) -> bool {
        self == Protocol::Tcp
    }

    pub fn is_udp(self) -> bool {
        self == Protocol::Udp
    }
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let protocol = match *self {
            Protocol::Tcp => "tcp",
            Protocol::Udp => "udp",
        };
        write!(f, "{}", protocol)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Url {
    pub host: String,
    pub port: Port,
    pub protocol: Protocol,
    address_cache: Address,
}

impl Url {
    pub fn new(address: Address, protocol: Protocol) -> Self {
        Self {
            host: address.ip().to_string(),
            port: address.port(),
            protocol,
            address_cache: address,
        }
    }

    /// NOTE: This function expects an input of the format: tcp://example.com:15600.
    pub async fn from_str(url: &str) -> Result<Self, Error> {
        if let Ok(url) = url::Url::parse(url) {
            let host = url.host_str().ok_or(Error::UrlDestructFailure)?.to_string();
            let port = Port(url.port().ok_or(Error::UrlDestructFailure)?);

            let address = &format!("{}:{}", host, *port)[..];
            let address = net::resolve_address(address)
                .await
                .map_err(|_| Error::ResolveFailure)?
                .into();

            let protocol = match url.scheme() {
                "tcp" => Protocol::Tcp,
                "udp" => Protocol::Udp,
                "" => return Err(Error::UnspecifiedProtocol),
                _ => return Err(Error::UnsupportedProtocol),
            };

            Ok(Self {
                host,
                port,
                protocol,
                address_cache: address,
            })
        } else {
            Err(Error::UrlParseFailure)
        }
    }

    pub async fn address(&mut self, update: bool) -> Result<Address, Error> {
        if update {
            let address = &format!("{}:{}", self.host, *self.port)[..];
            let address = net::resolve_address(address)
                .await
                .map_err(|_| Error::ResolveFailure)?
                .into();

            self.address_cache = address;
        }

        Ok(self.address_cache)
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}://{}:{}", self.protocol, self.host, *self.port)
    }
}

// TODO: test Error conditions
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_tcp_url_from_address_and_protocol() {
        let address = Url::from_str("tcp://127.0.0.1:15600")
            .unwrap()
            .address(true)
            .await
            .unwrap();

        let url = Url::new(address, Protocol::Tcp);

        assert_eq!(Protocol::Tcp, url.protocol);
        assert_eq!("tcp://127.0.0.1:15600", url.to_string());
    }

    #[tokio::test]
    async fn create_udp_url_from_address_and_protocol() {
        let address = Url::from_str("udp://127.0.0.1:15600")
            .unwrap()
            .address(true)
            .await
            .unwrap();

        let url = Url::new(address, Protocol::Udp);

        assert_eq!(Protocol::Udp, url.protocol);
        assert_eq!("udp://127.0.0.1:15600", url.to_string());
    }

    #[test]
    fn create_tcp_url_from_str() {
        let url = Url::from_str("tcp://127.0.0.1:15600");
        let url = url.expect("parsing url failed");

        assert_eq!(Protocol::Tcp, url.protocol);
        assert_eq!("tcp://127.0.0.1:15600", url.to_string());
    }

    #[test]
    fn create_udp_url_from_str() {
        let url = Url::from_str("udp://127.0.0.1:15600");
        let url = url.expect("parsing url failed");

        assert_eq!(Protocol::Udp, url.protocol);
        assert_eq!("udp://127.0.0.1:15600", url.to_string());
    }

    #[test]
    fn create_ipv6_url_from_str() {
        let mut url = Url::from_str("tcp://[::1]:15600").expect("parsing url failed");

        assert!(url.address(false).unwrap().is_ipv6());
        assert_eq!(Protocol::Tcp, url.protocol);
        assert_eq!("tcp://[::1]:15600", url.to_string());
    }
}
