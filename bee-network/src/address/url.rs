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

use super::{errors::*, Address};

use url::Url as ExternUrl;

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

/// Represents various types of `Url`s.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Url {
    /// A TCP Url.
    Tcp(Address),

    /// A UDP Url.
    Udp(Address),
}

const TCP: &str = "tcp";
const UDP: &str = "udp";

impl Url {
    /// Creates a new `Url`.
    pub fn new(addr: Address, proto: Protocol) -> Self {
        match proto {
            Protocol::Tcp => Url::Tcp(addr),
            Protocol::Udp => Url::Udp(addr),
        }
    }

    /// Creates a `Url` from a string slice.
    ///
    /// NOTE: This function expects an input of the format: tcp://example.com:15600.
    pub async fn from_url_str(url: &str) -> AddressResult<Self> {
        if let Ok(url) = ExternUrl::parse(url) {
            let host = url.host_str().ok_or(AddressError::UrlDestructFailure)?;
            let port = url.port().ok_or(AddressError::UrlDestructFailure)?;

            let host_port = &format!("{}:{}", host, port)[..];
            let addr = Address::from_addr_str(host_port).await?;

            match url.scheme() {
                TCP => Ok(Url::new(addr, Protocol::Tcp)),
                UDP => Ok(Url::new(addr, Protocol::Udp)),
                _ => Err(AddressError::UnsupportedProtocol),
            }
        } else {
            Err(AddressError::UrlParseFailure)
        }
    }

    /// Returns the `Address` of this `Url`.
    pub fn address(&self) -> Address {
        match *self {
            Url::Tcp(address) | Url::Udp(address) => address,
        }
    }

    /// Returns the `Protocol` of this `Url`.
    pub fn protocol(&self) -> Protocol {
        match *self {
            Url::Tcp(_) => Protocol::Tcp,
            Url::Udp(_) => Protocol::Udp,
        }
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Url::Tcp(ref address) => write!(f, "tcp://{}", address),
            Url::Udp(ref address) => write!(f, "udp://{}", address),
        }
    }
}

// TODO: test Error conditions
#[cfg(test)]
mod tests {
    use super::*;
    use async_std::task::block_on;

    #[test]
    fn create_tcp_url_from_address_and_protocol() {
        let addr = block_on(Address::from_addr_str("127.0.0.1:15600"));
        let addr = addr.expect("parsing address failed");

        let url = Url::new(addr, Protocol::Tcp);
        assert_eq!(Protocol::Tcp, url.protocol());
        assert_eq!("tcp://127.0.0.1:15600", url.to_string());
    }

    #[test]
    fn create_udp_url_from_address_and_protocol() {
        let addr = block_on(Address::from_addr_str("127.0.0.1:15600"));
        let addr = addr.expect("parsing address failed");

        let url = Url::new(addr, Protocol::Udp);
        assert_eq!(Protocol::Udp, url.protocol());
        assert_eq!("udp://127.0.0.1:15600", url.to_string());
    }

    #[test]
    fn create_tcp_url_from_str() {
        let url = block_on(Url::from_url_str("tcp://127.0.0.1:15600"));
        let url = url.expect("parsing url failed");

        assert_eq!(Protocol::Tcp, url.protocol());
        assert_eq!("tcp://127.0.0.1:15600", url.to_string());
    }

    #[test]
    fn create_udp_url_from_str() {
        let url = block_on(Url::from_url_str("udp://127.0.0.1:15600"));
        let url = url.expect("parsing url failed");

        assert_eq!(Protocol::Udp, url.protocol());
        assert_eq!("udp://127.0.0.1:15600", url.to_string());
    }

    #[test]
    fn create_ipv6_url_from_str() {
        let url = block_on(Url::from_url_str("tcp://[::1]:15600"));
        let url = url.expect("parsing url failed");

        assert!(url.address().is_ipv6());
        assert_eq!(Protocol::Tcp, url.protocol());
        assert_eq!("tcp://[::1]:15600", url.to_string());
    }
}
