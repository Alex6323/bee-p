use crate::errors::ConnectionResult as R;

use async_std::net::{
    SocketAddr,
    TcpStream,
};
use async_std::sync::Arc;

use std::fmt;

#[derive(Clone, Debug)]
pub enum Role {
    Client,
    Server,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Role::Client => write!(f, "Client"),
            Role::Server => write!(f, "Server"),
        }
    }
}

#[derive(Clone)]
pub struct TcpConnection {
    pub role: Role,
    pub local_addr: SocketAddr,
    pub remote_addr: SocketAddr,
    pub stream: Arc<TcpStream>,
}

impl TcpConnection {
    pub fn new(stream: TcpStream, role: Role) -> R<Self> {
        let local_addr = stream.local_addr()?;
        let remote_addr = stream.peer_addr()?;
        let stream = Arc::new(stream);

        Ok(Self {
            role,
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
        // NOTE: do we need more than this comparison?
        self.remote_addr.ip() == other.remote_addr.ip()
    }
}
