use crate::address::Address;
use crate::address::url::Url;
use crate::connection::ConnectionId;

use futures::channel::mpsc;

use std::fmt;

const EVENT_CHAN_CAPACITY: usize = 10000;

#[derive(Clone, Debug)]
pub enum Event {

    EndpointAdded {
        endpoint: ConnectionId,
        total: usize,
    },

    EndpointRemoved {
        endpoint: ConnectionId,
        total: usize,
    },

    EndpointAccepted {
        endpoint: ConnectionId,
        url: Url,
        //sender: BytesSender,
    },

    ConnectionEstablished {
        endpoint: ConnectionId,
        timestamp: u64,
        total: usize,
    },

    ConnectionDropped {
        endpoint: ConnectionId,
        total: usize,
    },

    /*
    // TODO: find better solution!
    SendRecvStopped {
        endpoint: ConnectionId,
    },
    */

    BytesSent {
        to: ConnectionId,
        num: usize,
    },

    BytesReceived {
        from: ConnectionId,
        with_addr: Address,
        num: usize,
        buffer: Vec<u8>,
    },

    TryConnect {
        to: ConnectionId,
        num_retries: Option<usize>,
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::EndpointAdded { endpoint, total } =>
                write!(f, "Event::EndpointAdded {{ ep = {:?}, total = {} }}", endpoint, total),

            Event::EndpointRemoved { endpoint, total } =>
                write!(f, "Event::EndpointRemoved {{ ep = {:?}, total = {} }}", endpoint, total),

            Event::EndpointAccepted { endpoint, url, .. } =>
                write!(f, "Event::EndpointAccepted {{ ep = {:?}, url = {} }}", endpoint, url.to_string()),

            Event::ConnectionEstablished { endpoint, timestamp, total } =>
                write!(f, "Event::ConnectionEstablished {{ ep = {:?}, timestamp = {}, total = {} }}", endpoint, timestamp, total),

            Event::ConnectionDropped { endpoint, total } =>
                write!(f, "Event::ConnectionDropped {{ ep = {:?}, total = {} }}", endpoint, total),

            Event::BytesSent { to, num } =>
                write!(f, "Event::BytesSent {{ to = {}, num = {} }}", to, num),

            Event::BytesReceived { from, with_addr, num, .. } =>
                write!(f, "Event::BytesReceived {{ from = {}, with_addr = {}, num = {} }}", from, with_addr, num),

            Event::TryConnect { to, num_retries } =>
                write!(f, "Event::TryConnect {{ to = {}, num_retries = {:?} }}", to, num_retries),

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
