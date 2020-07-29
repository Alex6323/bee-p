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

use async_std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};

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

    #[error("Unsupported protocol.")]
    UnsupportedProtocol,

    #[error("Error resolving domain name to address.")]
    ResolveFailure,
}

/// A wrapper around a `u16` describing a network port number to increase type safety.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Port(pub u16);

impl ops::Deref for Port {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A wrapper around a socket address.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Address {
    inner: SocketAddr,
}

impl Address {
    /// Creates an `Address` from an `Ipv4Addr` and a `Port`.
    pub fn from_v4_addr_and_port(address: Ipv4Addr, port: Port) -> Self {
        Self {
            inner: SocketAddr::new(IpAddr::V4(address), *port),
        }
    }

    /// Creates an `Address` from an `Ipv6Addr` and a `Port`.
    pub fn from_v6_addr_and_port(address: Ipv6Addr, port: Port) -> Self {
        Self {
            inner: SocketAddr::new(IpAddr::V6(address), *port),
        }
    }

    /// Creates an `Address` from a host address string (e.g. "example.com:15600").
    ///
    /// NOTE: This operation is async, and can fail if the host name can't be resolved.
    pub async fn from_addr_str(address: &str) -> Result<Self, Error> {
        let address = address.to_socket_addrs().await?.next();

        match address {
            Some(address) => Ok(address.into()),
            None => Err(Error::ResolveFailure),
        }
    }

    /// Returns the port number of this address.
    pub fn port(&self) -> Port {
        Port(self.inner.port())
    }

    /// Returns the underlying IPv4 or IPv6 address.
    pub fn ip(&self) -> IpAddr {
        self.inner.ip()
    }

    /// Returns whether `self` represents a V4 address.
    pub fn is_ipv4(&self) -> bool {
        self.inner.is_ipv4()
    }

    /// Returns whether `self` represents a V6 address.
    pub fn is_ipv6(&self) -> bool {
        self.inner.is_ipv6()
    }
}

impl From<SocketAddr> for Address {
    fn from(inner: SocketAddr) -> Self {
        Self { inner }
    }
}

impl ops::Deref for Address {
    type Target = SocketAddr;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::task::block_on;

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

    #[test]
    fn create_address_from_str() {
        let address = block_on(Address::from_addr_str("localhost:15600"));
        assert!(address.is_ok());

        let address = address.unwrap();
        assert!(address.is_ipv4() || address.is_ipv6());
        assert_eq!(15600, *address.port());
    }

    #[test]
    fn create_address_without_port_should_panic() {
        let address = block_on(Address::from_addr_str("localhost"));
        assert!(address.is_err());
    }
}
