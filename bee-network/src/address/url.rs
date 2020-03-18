use super::errors::*;
use super::Address;

use std::fmt;

const PROTOCOL_SEPARATOR: &'static str = "://";
const PORT_SEPARATOR: &'static str = ":";

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Url {
    Tcp(Address),
    Udp(Address),
}

impl Url {
    pub fn new(addr: Address, proto: Protocol) -> Self {
        match proto {
            Protocol::Tcp => Url::Tcp(addr),
            Protocol::Udp => Url::Udp(addr),
        }
    }

    /// Example: tcp://example.com:15600
    pub async fn from_str_with_port(url: &str) -> AddressResult<Self> {
        // TODO: should be use 'url' crate instead of manual parsing?
        let proto_addr: Vec<&str> = url.split_terminator(PROTOCOL_SEPARATOR).collect();
        if proto_addr.len() != 2 {
            return Err(AddressError::UrlParseFailure);
        }

        let protocol = proto_addr[0];
        let host_addr = proto_addr[1];

        let hostname_port: Vec<&str> = host_addr.rsplit_terminator(PORT_SEPARATOR).collect();
        if hostname_port.len() != 2 {
            //TODO
            //u16::try_parse(hostname_port[hostname_port.len() - 1]).is_ok();
            return Err(AddressError::UrlParseFailure);
        }

        let addr = Address::from_host_addr(host_addr).await?;

        match protocol {
            "tcp" => Ok(Url::new(addr, Protocol::Tcp)),
            "udp" => Ok(Url::new(addr, Protocol::Udp)),
            _ => Err(AddressError::UrlParseFailure),
        }
    }

    pub fn address(&self) -> &Address {
        match *self {
            Url::Tcp(ref address) | Url::Udp(ref address) => address,
        }
    }

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
    fn create_tcp_url_from_addr_and_protocol() {
        let addr = block_on(Address::from_host_addr("localhost:15600"));
        assert!(addr.is_ok());

        let addr = addr.unwrap();
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
        let addr = block_on(Address::from_host_addr("localhost:15600"));
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
}
