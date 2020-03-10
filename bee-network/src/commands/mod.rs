use crate::address::Address;
use crate::connection::ConnectionId;

use futures::channel::oneshot;

use std::fmt;

pub type Responder<T> = oneshot::Sender<T>;
pub type Requester<T> = oneshot::Receiver<T>;

#[derive(Debug)]
pub enum Command {

    AddEndpoint {
        address: Address,
        responder: Responder<Option<ConnectionId>>,
    },

    RemoveEndpoint {
        conn: ConnectionId,
        responder: Responder<bool>,
    },

    Connect {
        conn: ConnectionId,
        attempts: Option<usize>,
        responder: Responder<bool>,
    },

    Disconnect {
        conn: ConnectionId,
        responder: Responder<bool>,
    },

    UnicastBytes {
        conn: ConnectionId,
        bytes: Vec<u8>,
    },

    MulticastBytes {
        conns: Vec<ConnectionId>,
        bytes: Vec<u8>,
    },

    BroadcastBytes {
        bytes: Vec<u8>,
    },

    Shutdown {
        responder: Responder<bool>,
    },
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::AddEndpoint { address, .. } =>
                write!(f, "Command::AddEndpoint {{ address = {:?} }} ", address),

            Command::RemoveEndpoint { conn, .. } =>
                write!(f, "Command::RemoveEndpoint {{ conn = {:?} }}", conn),

            Command::Connect { conn, attempts, .. } =>
                write!(f, "Command::Connect {{ conn = {:?}, attempts = {:?} }}", conn, attempts),

            Command::Disconnect { conn, .. } =>
                write!(f, "Command::Disconnect {{ conn = {:?} }}", conn),

            Command::UnicastBytes { conn, .. } =>
                write!(f, "Command::UnicastBytes {{ conn = {:?} }}", conn),

            Command::MulticastBytes { conns, .. } =>
                write!(f, "Command::MulticastBytes {{ num_conns = {} }}", conns.len()),

            Command::BroadcastBytes { .. } =>
                write!(f, "Command::BroadcastBytes"),

            Command::Shutdown { .. }=>
                write!(f, "Command::Shutdown"),
        }
    }
}