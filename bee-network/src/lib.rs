use std::net::{SocketAddr, ToSocketAddrs};
use tokio::net::TcpListener;

pub struct Peer {
    address: SocketAddr,
}

impl Peer {
    pub fn new(address: impl ToSocketAddrs) -> Self {
        Peer {
            address: address.to_socket_addrs().unwrap().next().unwrap(),
        }
    }
}
