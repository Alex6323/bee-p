use crate::{
    endpoint::origin::Origin,
    errors::ConnectionResult,
};

use async_std::{
    net::{
        SocketAddr,
        TcpStream,
    },
    sync::Arc,
};

use std::fmt;

#[derive(Clone)]
pub struct TcpConnection {
    pub origin: Origin,
    pub local_addr: SocketAddr,
    pub remote_addr: SocketAddr,
    pub stream: Arc<TcpStream>,
}

impl TcpConnection {
    pub fn new(stream: TcpStream, origin: Origin) -> ConnectionResult<Self> {
        let local_addr = stream.local_addr()?;
        let remote_addr = stream.peer_addr()?;
        let stream = Arc::new(stream);

        Ok(Self {
            origin,
            local_addr,
            remote_addr,
            stream,
        })
    }
}

impl fmt::Display for TcpConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} <-> {}", self.local_addr, self.remote_addr)
    }
}

impl Eq for TcpConnection {}
impl PartialEq for TcpConnection {
    fn eq(&self, other: &Self) -> bool {
        // TODO: use socket address instead of IP
        self.remote_addr.ip() == other.remote_addr.ip()
    }
}
