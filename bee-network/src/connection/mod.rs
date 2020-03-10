pub mod pool;

use crate::address::Address;
use crate::address::url::{Protocol, Url};

use std::fmt;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ConnectionId {
    inner: Address,
}

impl From<Address> for ConnectionId {
    fn from(addr: Address) -> Self {
        Self { inner: addr }
    }
}

impl From<Url> for ConnectionId {
    fn from(url: Url) -> Self {
        match url {
            Url::Tcp(socket_addr) | Url::Udp(socket_addr) => Self { inner: socket_addr.into() },
        }
    }
}

impl fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ConnectionState {
    Awaited,
    Established,
    Broken,
    Stalled,
}

#[derive(Debug)]
pub struct Connection {
    pub id: ConnectionId,
    pub endpoint: Address,
    pub protocol: Protocol,
    pub state: ConnectionState,
}

impl Connection {
    pub fn from_url(url: Url) -> Self {
        let endpoint = url.address().clone();
        let protocol = url.protocol();

        Self {
            id: url.into(),
            endpoint,
            protocol,
            state: ConnectionState::Awaited,
        }
    }

    pub fn is_awaited(&self) -> bool {
        self.state == ConnectionState::Awaited
    }

    pub fn is_established(&self) -> bool {
        self.state == ConnectionState::Established
    }

    pub fn is_broken(&self) -> bool {
        self.state == ConnectionState::Broken
    }

    pub fn is_stalled(&self) -> bool {
        self.state == ConnectionState::Stalled
    }
}

impl Eq for Connection {}
impl PartialEq for Connection {
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

        let conn_id: ConnectionId = addr.into();

        assert_eq!("127.0.0.1:16000", conn_id.to_string());
    }

    #[test]
    fn create_conn_id_from_url() {
        let url = block_on(Url::from_str_with_port("tcp://localhost:16000")).unwrap();

        let conn_id: ConnectionId = url.into();

        assert_eq!("127.0.0.1:16000", conn_id.to_string());
    }

    #[test]
    fn create_conn_from_url() {
        let url = block_on(Url::from_str_with_port("udp://localhost:16000")).unwrap();
        let conn = Connection::from_url(url);

        assert!(conn.is_awaited());
        assert_eq!("127.0.0.1:16000", conn.id.to_string());
        assert_eq!(Protocol::Udp, conn.protocol);
        assert_eq!("127.0.0.1:16000", conn.endpoint.to_string());
    }
}