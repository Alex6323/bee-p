use crate::address::Address;
use crate::commands::Responder;
use crate::endpoint::{
    outbox::BytesSender,
    Endpoint,
    EndpointId,
};

use futures::channel::mpsc;
use std::fmt;

// TODO: remove this
const EVENT_CHAN_CAPACITY: usize = 10000;

#[derive(Debug)]
pub enum Event {
    EndpointAdded {
        epid: EndpointId,
        total: usize,
    },

    EndpointRemoved {
        epid: EndpointId,
        total: usize,
    },

    NewConnection {
        ep: Endpoint,
        sender: BytesSender,
    },

    LostConnection {
        epid: EndpointId,
    },

    EndpointConnected {
        epid: EndpointId,
        timestamp: u64,
        total: usize,
    },

    EndpointDisconnected {
        epid: EndpointId,
        total: usize,
    },

    BytesSent {
        epid: EndpointId,
        num: usize,
    },

    BytesReceived {
        epid: EndpointId,
        addr: Address,
        bytes: Vec<u8>,
    },

    TryConnect {
        epid: EndpointId,
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

            Event::EndpointConnected { epid, timestamp, total } => write!(
                f,
                "Event::EndpointConnected {{ {}, ts: {}, num_connected: {} }}",
                epid, timestamp, total
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
pub type EventSubscriber = mpsc::Receiver<Event>;

// TODO: what's a good value here?
// TODO: move this into `constants.rs`
const EVENT_CHANNEL_CAPACITY: usize = 10000;

pub fn event_channel() -> (EventPublisher, EventSubscriber) {
    mpsc::channel(EVENT_CHANNEL_CAPACITY)
}
