use std::collections::HashSet;
use std::ops::Range;

use async_std::net::SocketAddr;

pub trait Message {
    type Error;

    fn size_range() -> Range<usize>
    where
        Self: std::marker::Sized;

    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Error>
    where
        Self: std::marker::Sized;

    fn into_bytes(self) -> Vec<u8>;
}

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
