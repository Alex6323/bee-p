use std::collections::HashSet;
use std::ops::Range;

use async_std::io::Read;
use async_std::net::SocketAddr;
use async_trait::async_trait;

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

#[async_trait]
pub trait MessageReader {
    type MessageType;
    type Error;

    async fn read<R>(reader: R) -> Result<Self::MessageType, Self::Error>
    where
        R: Read + std::marker::Unpin + std::marker::Send;
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
