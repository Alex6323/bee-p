use std::collections::HashSet;
use std::fmt;
use std::io::Error;
use std::io::ErrorKind;

use async_std::net::SocketAddr;

use bee_protocol::MessageType;

#[derive(Clone)]
pub struct MessageToSend {
    pub to: HashSet<SocketAddr>,
    pub msg: MessageType,
}

#[derive(Clone)]
pub struct ReceivedMessage {
    pub from: SocketAddr,
    pub msg: MessageType,
}
