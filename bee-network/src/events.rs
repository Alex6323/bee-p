use crate::address::url::Url;
use crate::address::Address;
use crate::commands::Responder;
use crate::connection::BytesSender;
use crate::endpoint::EndpointId;

use futures::channel::mpsc;

use std::fmt;

const EVENT_CHAN_CAPACITY: usize = 10000;

#[derive(Debug)]
pub enum Event {
    EndpointAdded {
        id: EndpointId,
        total: usize,
    },

    EndpointRemoved {
        id: EndpointId,
        total: usize,
    },

    EndpointAccepted {
        id: EndpointId,
        url: Url,
        sender: BytesSender,
    },

    ConnectionEstablished {
        id: EndpointId,
        timestamp: u64,
        total: usize,
    },

    ConnectionDropped {
        id: EndpointId,
        total: usize,
    },

    BytesSent {
        to: EndpointId,
        num: usize,
    },

    BytesReceived {
        from: EndpointId,
        with_addr: Address,
        num: usize,
        buffer: Vec<u8>,
    },

    TryConnect {
        to: EndpointId,
        responder: Option<Responder<bool>>,
    },
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::EndpointAdded { id, total } => {
                write!(f, "Event::EndpointAdded {{ id = {:?}, total = {} }}", id, total)
            }

            Event::EndpointRemoved { id, total } => {
                write!(f, "Event::EndpointRemoved {{ id = {:?}, total = {} }}", id, total)
            }

            Event::EndpointAccepted { id, url, .. } => write!(
                f,
                "Event::EndpointAccepted {{ id = {:?}, url = {} }}",
                id,
                url.to_string()
            ),

            Event::ConnectionEstablished { id, timestamp, total } => write!(
                f,
                "Event::ConnectionEstablished {{ id = {:?}, timestamp = {}, total = {} }}",
                id, timestamp, total
            ),

            Event::ConnectionDropped { id, total } => {
                write!(f, "Event::ConnectionDropped {{ id = {:?}, total = {} }}", id, total)
            }

            Event::BytesSent { to, num } => write!(f, "Event::BytesSent {{ to = {}, num = {} }}", to, num),

            Event::BytesReceived {
                from, with_addr, num, ..
            } => write!(
                f,
                "Event::BytesReceived {{ from = {}, with_addr = {}, num = {} }}",
                from, with_addr, num
            ),

            Event::TryConnect { to, .. } => write!(f, "Event::TryConnect {{ to = {} }}", to),
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
