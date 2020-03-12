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
        epid: EndpointId,
        total: usize,
    },

    EndpointRemoved {
        epid: EndpointId,
        total: usize,
    },

    EndpointAccepted {
        epid: EndpointId,
        url: Url,
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
            Event::EndpointAdded { epid, total } => {
                write!(f, "Event::EndpointAdded {{ epid = {:?}, total = {} }}", epid, total)
            }

            Event::EndpointRemoved { epid, total } => {
                write!(f, "Event::EndpointRemoved {{ epid = {:?}, total = {} }}", epid, total)
            }

            Event::EndpointAccepted { epid, url, .. } => write!(
                f,
                "Event::EndpointAccepted {{ epid = {:?}, url = {} }}",
                epid,
                url.to_string()
            ),

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
