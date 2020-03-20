pub mod errors;
pub mod url;

use errors::*;

use async_std::net::{
    IpAddr,
    Ipv4Addr,
    Ipv6Addr,
    SocketAddr,
    ToSocketAddrs,
};

use std::fmt;
use std::ops;

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
    pub fn from_v4_addr_and_port(addr: Ipv4Addr, port: Port) -> Self {
        Self {
            inner: SocketAddr::new(IpAddr::V4(addr), *port),
        }
    }

    /// Creates an `Address` from an `Ipv6Addr` and a `Port`.
    pub fn from_v6_addr_and_port(addr: Ipv6Addr, port: Port) -> Self {
        Self {
            inner: SocketAddr::new(IpAddr::V6(addr), *port),
        }
    }

    /// Creates an `Address` from a host address string (e.g. "example.com:15600").
    ///
    /// NOTE: This operation is async, and can fail if the host name can't be resolved.
    pub async fn from_host_addr(host_addr: &str) -> AddressResult<Self> {
        let address = host_addr.to_socket_addrs().await?.next();

        match address {
            Some(address) => Ok(address.into()),
            None => Err(AddressError::ResolveFailure),
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
        assert_eq!(Port(15600), address.port());
    }

    #[test]
    fn create_address_from_v6_addr() {
        let address = Address::from_v6_addr_and_port(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), Port(15600));

        assert!(address.is_ipv6());
        assert_eq!("[::1]:15600", address.to_string());
        assert_eq!(Port(15600), address.port());
    }

    #[test]
    fn create_address_from_v4_socket_addr() {
        let address: Address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), *Port(15600)).into();

        assert!(address.is_ipv4());
        assert_eq!("127.0.0.1:15600", address.to_string());
        assert_eq!(Port(15600), address.port());
    }

    #[test]
    fn create_address_from_v6_socket_addr() {
        let address: Address = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), *Port(15600)).into();

        assert!(address.is_ipv6());
        assert_eq!("[::1]:15600", address.to_string());
        assert_eq!(Port(15600), address.port());
    }

    #[test]
    fn create_address_from_hostname() {
        let address = block_on(Address::from_host_addr("localhost:15600"));
        assert!(address.is_ok());

        let address = address.unwrap();
        assert!(address.is_ipv4());
        assert_eq!("127.0.0.1:15600", address.to_string());
        assert_eq!(Port(15600), address.port());
    }
}
