use crate::address::Address;
use crate::commands::Responder;
use crate::endpoint::role::Role;
use crate::endpoint::{
    outbox::BytesSender,
    Endpoint,
    EndpointId,
};

use futures::channel::mpsc;
use std::fmt;

/// Network events.
#[derive(Debug)]
pub enum Event {
    /// Signals that a new `Endpoint` has been added.
    EndpointAdded {
        /// The id of new `Endpoint`.
        epid: EndpointId,

        /// The total number of managed `Endpoint`s.
        total: usize,
    },

    /// Signals that an  `Endpoint` has been removed.
    EndpointRemoved {
        /// The id of the removed `Endpoint`.
        epid: EndpointId,

        /// The total number of the remaining `Endpoint`s.
        total: usize,
    },

    /// Signals that a new connection was established.
    NewConnection {
        /// The new `Endpoint`.
        ep: Endpoint,

        /// The connection relationship with the endpoint. Can either be
        /// `Role::Server` (incoming) or `Role::Client` (outgoing).
        role: Role,

        /// The channel half to send messages over this connection.
        sender: BytesSender,
    },

    /// Signals that a connection has been dropped.
    LostConnection {
        /// The id of the previously connected connections.
        epid: EndpointId,
    },

    /// Signals that a connection to an `Endpoint` has been established.
    EndpointConnected {
        /// The id of the connected `Endpoint`.
        epid: EndpointId,

        /// The address of the connected endpoint.
        address: Address,

        /// The connection relationship with the endpoint. Can either be
        /// `Role::Server` (incoming) or `Role::Client` (outgoing).
        role: Role,

        /// The timestamp when the connection was established.
        timestamp: u64,

        /// The total number of active connections.
        total: usize,
    },

    /// Signals that a connection to an `Endpoint` has been dropped.
    EndpointDisconnected {
        /// The id of the disconnected `Endpoint`.
        epid: EndpointId,

        /// The total number of remaining connections.
        total: usize,
    },

    // TODO: rename to `MessageSent`
    /// Signals that a message has been sent.
    BytesSent {
        /// The id of the `Endpoint` a message was sent to.
        epid: EndpointId,

        // TODO: rename to `num_bytes`
        /// The number of bytes sent.
        num: usize,
    },

    // TODO: rename to `MessageReceived`
    /// Signals that a message has been received.
    BytesReceived {
        /// The id of the `Endpoint` a message was received from.
        epid: EndpointId,

        // TODO: remove this for now.
        /// The `Address` of the `Endpoint` a message was received from.
        addr: Address,

        /// The raw bytes of the message.
        bytes: Vec<u8>,
    },

    /// Signals the next connection attempt to an `Endpoint`.
    TryConnect {
        /// The id of the `Endpoint`.
        epid: EndpointId,

        /// The success responder.
        responder: Option<Responder<bool>>,
    },
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::EndpointAdded { epid, total } => {
                write!(f, "Event::EndpointAdded {{ {}, num_endpoints: {} }}", epid, total)
            }

            Event::EndpointRemoved { epid, total } => {
                write!(f, "Event::EndpointRemoved {{ {}, num_endpoints: {} }}", epid, total)
            }

            Event::NewConnection { ep, .. } => write!(f, "Event::NewConnection {{ {} }}", ep.id,),
            Event::LostConnection { epid, .. } => write!(f, "Event::LostConnection {{ {} }}", epid,),

            Event::EndpointConnected {
                epid,
                address,
                role,
                timestamp,
                total,
            } => write!(
                f,
                "Event::EndpointConnected {{ {}, address: {}, role: {}, ts: {}, num_connected: {} }}",
                epid, address, role, timestamp, total
            ),

            Event::EndpointDisconnected { epid, total } => write!(
                f,
                "Event::EndpointDisconnected {{ {}, num_connected: {} }}",
                epid, total
            ),

            Event::BytesSent { epid, num } => write!(f, "Event::BytesSent {{ {}, num_bytes: {} }}", epid, num),

            Event::BytesReceived { epid, addr, bytes } => write!(
                f,
                "Event::BytesReceived {{ {}, from: {}, num_bytes: {} }}",
                epid,
                addr,
                bytes.len()
            ),

            Event::TryConnect { epid, .. } => write!(f, "Event::TryConnect {{ {} }}", epid),
        }
    }
}

pub type EventPublisher = mpsc::Sender<Event>;

// TODO: create a wrapper type to not expose futures::mpsc directly.
/// `Event` receiver channel half.
pub type EventSubscriber = mpsc::Receiver<Event>;

// TODO: what's a good value here?
// TODO: move this into `constants.rs`
const EVENT_CHANNEL_CAPACITY: usize = 10000;

pub fn event_channel() -> (EventPublisher, EventSubscriber) {
    mpsc::channel(EVENT_CHANNEL_CAPACITY)
}
