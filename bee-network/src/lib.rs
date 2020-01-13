mod constants;

use std::net::{SocketAddr, ToSocketAddrs};
use tokio::net::TcpListener;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PeerId(pub SocketAddr);

pub struct Peer {
    address: SocketAddr,
}

impl Peer {
    pub fn new(address: impl ToSocketAddrs) -> Self {
        Peer {
            address: address.to_socket_addrs().unwrap().next().unwrap(),
        }
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }
}
