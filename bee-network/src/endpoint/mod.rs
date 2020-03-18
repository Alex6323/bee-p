pub mod actor;
pub mod outbox;
pub mod store;

use crate::address::url::{
    Protocol,
    Url,
};
use crate::address::Address;

use std::fmt;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct EndpointId {
    inner: Address,
}

impl From<Address> for EndpointId {
    fn from(addr: Address) -> Self {
        Self { inner: addr }
    }
}

impl From<Url> for EndpointId {
    fn from(url: Url) -> Self {
        match url {
            Url::Tcp(socket_addr) | Url::Udp(socket_addr) => Self {
                inner: socket_addr.into(),
            },
        }
    }
}

impl fmt::Display for EndpointId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum EndpointState {
    Disconnected,
    Connected,
}

#[derive(Clone, Debug)]
pub struct Endpoint {
    pub id: EndpointId,
    pub address: Address,
    pub protocol: Protocol,
    //pub state: EndpointState,
}

impl Endpoint {
    pub fn new(address: Address, protocol: Protocol) -> Self {
        Self {
            id: address.clone().into(),
            address,
            protocol,
            //state: EndpointState::Disconnected,
        }
    }
    pub fn from_url(url: Url) -> Self {
        let address = url.address().clone();
        let protocol = url.protocol();

        Endpoint::new(address, protocol)
    }

    /*
    pub fn is_connected(&self) -> bool {
        self.state == EndpointState::Connected
    }

    pub fn is_disconnected(&self) -> bool {
        self.state == EndpointState::Disconnected
    }

    pub fn set_connected(&mut self) {
        self.state = EndpointState::Connected;
    }

    pub fn set_disconnected(&mut self) {
        self.state = EndpointState::Disconnected;
    }
    */
}

impl Eq for Endpoint {}
impl PartialEq for Endpoint {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::task::block_on;

    #[test]
    fn create_conn_id_from_address() {
        let addr = block_on(Address::from_host_addr("localhost:16000")).unwrap();

        let conn_id: EndpointId = addr.into();

        assert_eq!("127.0.0.1:16000", conn_id.to_string());
    }

    #[test]
    fn create_conn_id_from_url() {
        let url = block_on(Url::from_str_with_port("tcp://localhost:16000")).unwrap();

        let conn_id: EndpointId = url.into();

        assert_eq!("127.0.0.1:16000", conn_id.to_string());
    }

    #[test]
    fn create_conn_from_url() {
        let url = block_on(Url::from_str_with_port("udp://localhost:16000")).unwrap();
        let ep = Endpoint::from_url(url);

        assert!(ep.is_disconnected());
        assert_eq!("127.0.0.1:16000", ep.id.to_string());
        assert_eq!(Protocol::Udp, ep.protocol);
        assert_eq!("127.0.0.1:16000", ep.address.to_string());
    }
}
