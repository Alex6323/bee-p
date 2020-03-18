use crate::endpoint::EndpointId as EpId;

use async_std::net::{
    SocketAddr,
    UdpSocket,
};

pub struct UdpConnection {
    pub epid: EpId,
    pub remote_addr: SocketAddr,
    socket: UdpSocket,
}
