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

pub mod url;

use async_std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use std::{fmt, ops};

use thiserror::Error;

/// Errors that can happen when dealing with `Address`es.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Error resolving domain name to address.")]
    Io(#[from] std::io::Error),

    #[error("Error parsing url.")]
    UrlParseFailure,

    #[error("Error destructing url.")]
    UrlDestructFailure,

    #[error("Unspecified protocol.")]
    UnspecifiedProtocol,

    #[error("Unsupported protocol.")]
    UnsupportedProtocol,

    #[error("Error resolving domain name to address.")]
    ResolveFailure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Port(u16);

impl Port {
    pub fn new(port: u16) -> Self {
        if port < 1024 {
            panic!("Invalid port number");
        }

        Self(port)
    }
}

impl ops::Deref for Port {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Address(SocketAddr);

impl Address {
    pub fn from_str(address: &str) -> Result<Self, Error> {
        if let Ok(address) = address.parse::<SocketAddr>() {
            Ok(Self(address))
        } else {
            if let Ok(address) = crate::utils::net::resolve_address(address) {
                Ok(Self(address))
            } else {
                Err(Error::ResolveFailure)
            }
        }
    }

    pub fn from_v4_addr_and_port(address: Ipv4Addr, port: Port) -> Self {
        Self(SocketAddr::new(IpAddr::V4(address), *port))
    }

    pub fn from_v6_addr_and_port(address: Ipv6Addr, port: Port) -> Self {
        Self(SocketAddr::new(IpAddr::V6(address), *port))
    }

    pub fn port(&self) -> Port {
        Port(self.0.port())
    }

    pub fn ip(&self) -> IpAddr {
        self.0.ip()
    }

    pub fn is_ipv4(&self) -> bool {
        self.0.is_ipv4()
    }

    pub fn is_ipv6(&self) -> bool {
        self.0.is_ipv6()
    }
}

impl From<SocketAddr> for Address {
    fn from(inner: SocketAddr) -> Self {
        Self(inner)
    }
}

impl ops::Deref for Address {
    type Target = SocketAddr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_address_from_str() {
        let address = Address::from_str("localhost:15600").expect("parse error");

        assert!(address.is_ipv4() || address.is_ipv6());
        assert_eq!(15600, *address.port());
    }

    #[test]
    fn create_address_without_port_should_panic() {
        let address = Address::from_str("localhost");
        assert!(address.is_err());
    }

    #[test]
    fn create_address_from_v4_addr() {
        let address = Address::from_v4_addr_and_port(Ipv4Addr::new(127, 0, 0, 1), Port(15600));

        assert!(address.is_ipv4());
        assert_eq!("127.0.0.1:15600", address.to_string());
        assert_eq!(15600, *address.port());
    }

    #[test]
    fn create_address_from_v6_addr() {
        let address = Address::from_v6_addr_and_port(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), Port(15600));

        assert!(address.is_ipv6());
        assert_eq!("[::1]:15600", address.to_string());
        assert_eq!(15600, *address.port());
    }

    #[test]
    fn create_address_from_v4_socket_addr() {
        let address: Address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 15600).into();

        assert!(address.is_ipv4());
        assert_eq!("127.0.0.1:15600", address.to_string());
        assert_eq!(15600, *address.port());
    }

    #[test]
    fn create_address_from_v6_socket_addr() {
        let address: Address = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 15600).into();

        assert!(address.is_ipv6());
        assert_eq!("[::1]:15600", address.to_string());
        assert_eq!(15600, *address.port());
    }
}
