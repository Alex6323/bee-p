use super::{
    errors::*,
    Address,
};

use url::Url as ExternUrl;

use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Protocol {
    Tcp,
    Udp,
}

impl Protocol {
    pub fn is_tcp(&self) -> bool {
        *self == Protocol::Tcp
    }

    pub fn is_udp(&self) -> bool {
        *self == Protocol::Udp
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

const TCP: &'static str = "tcp";
const UDP: &'static str = "udp";

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
    pub async fn from_str_with_port(url: &str) -> AddressResult<Self> {
        if let Ok(url) = ExternUrl::parse(url) {
            let host = url.host_str().ok_or(AddressError::UrlDestructFailure)?;
            let port = url.port().ok_or(AddressError::UrlDestructFailure)?;

            let host_port = &format!("{}:{}", host, port)[..];
            let addr = Address::from_host_addr(host_port).await?;

            match url.scheme() {
                TCP => Ok(Url::new(addr, Protocol::Tcp)),
                UDP => Ok(Url::new(addr, Protocol::Udp)),
                _ => Err(AddressError::UnsupportedProtocol),
            }
        } else {
            return Err(AddressError::UrlParseFailure);
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
        let addr = block_on(Address::from_host_addr("127.0.0.1:15600"));
        assert!(addr.is_ok());

        let addr = addr.unwrap();
        assert!(addr.is_ipv4());

        let url = Url::new(addr, Protocol::Tcp);
        assert_eq!(Protocol::Tcp, url.protocol());
        assert_eq!("127.0.0.1:15600", url.address().to_string());
        assert_eq!("tcp://127.0.0.1:15600", url.to_string());
    }

    #[test]
    fn create_tcp_url_from_str_with_port() {
        let url = block_on(Url::from_str_with_port("tcp://127.0.0.1:15600"));
        assert!(url.is_ok());

        let url = url.unwrap();
        assert_eq!(Protocol::Tcp, url.protocol());
        assert_eq!("127.0.0.1:15600", url.address().to_string());
        assert_eq!("tcp://127.0.0.1:15600", url.to_string());
    }

    #[test]
    fn create_udp_url_from_addr_and_protocol() {
        let addr = block_on(Address::from_host_addr("127.0.0.1:15600"));
        assert!(addr.is_ok());

        let addr = addr.unwrap();
        let url = Url::new(addr, Protocol::Udp);

        assert_eq!(Protocol::Udp, url.protocol());
        assert_eq!("127.0.0.1:15600", url.address().to_string());
        assert_eq!("udp://127.0.0.1:15600", url.to_string());
    }

    #[test]
    fn create_udp_url_from_str_with_port() {
        let url = block_on(Url::from_str_with_port("udp://127.0.0.1:15600"));
        assert!(url.is_ok());

        let url = url.unwrap();
        assert_eq!(Protocol::Udp, url.protocol());
        assert_eq!("127.0.0.1:15600", url.address().to_string());
        assert_eq!("udp://127.0.0.1:15600", url.to_string());
    }

    #[test]
    fn create_tcp_ipv6_url_from_str_with_port() {
        let url = block_on(Url::from_str_with_port("tcp://[::1]:15600"));
        assert!(url.is_ok());

        let url = url.unwrap();
        assert_eq!(Protocol::Tcp, url.protocol());
        assert_eq!("[::1]:15600", url.address().to_string());
        assert_eq!("tcp://[::1]:15600", url.to_string());
    }
}
