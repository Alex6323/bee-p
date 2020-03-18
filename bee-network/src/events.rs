use crate::address::Address;
use crate::commands::Responder;
use crate::endpoint::{
    outbox::BytesSender,
    Endpoint,
    EndpointId,
};

use futures::channel::mpsc;
use std::fmt;

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
                write!(f, "Event::EndpointAdded {{ epid = {:?}, total = {} }}", epid, total)
            }

            Event::EndpointRemoved { epid, total } => {
                write!(f, "Event::EndpointRemoved {{ epid = {:?}, total = {} }}", epid, total)
            }

            Event::NewConnection { ep, .. } => write!(f, "Event::NewConnection {{ epid = {:?} }}", ep.id,),

            Event::EndpointConnected { epid, timestamp, total } => write!(
                f,
                "Event::EndpointConnected {{ epid = {:?}, timestamp = {}, total = {} }}",
                epid, timestamp, total
            ),

            Event::EndpointDisconnected { epid, total } => write!(
                f,
                "Event::EndpointDisconnected {{ epid = {:?}, total = {} }}",
                epid, total
            ),

            Event::BytesSent { epid, num } => write!(f, "Event::BytesSent {{ epid = {}, num = {} }}", epid, num),

            Event::BytesReceived { epid, addr, bytes } => write!(
                f,
                "Event::BytesReceived {{ epid = {}, addr = {}, num = {} }}",
                epid,
                addr,
                bytes.len()
            ),

            Event::TryConnect { epid, .. } => write!(f, "Event::TryConnect {{ epid = {} }}", epid),
        }
    }
}

pub type EventPublisher = mpsc::Sender<Event>;
pub type EventSubscriber = mpsc::Receiver<Event>;

// TODO: what's a good value here?
const EVENT_CHANNEL_CAPACITY: usize = 10000;

pub fn event_channel() -> (EventPublisher, EventSubscriber) {
    mpsc::channel(EVENT_CHANNEL_CAPACITY)
}
