use std::collections::HashSet;

use async_std::io::Read;
use async_std::net::SocketAddr;
use async_trait::async_trait;

// TODO M bounds ? Deref ?

#[derive(Clone)]
pub struct MessageToSend<M> {
    pub to: HashSet<SocketAddr>,
    pub msg: M,
}

#[derive(Clone)]
pub struct ReceivedMessage<M> {
    pub from: SocketAddr,
    pub msg: M,
}
