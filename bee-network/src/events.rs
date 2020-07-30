// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use crate::{
    address::Address,
    endpoint::{origin::Origin, outbox::BytesSender, Endpoint, EndpointId},
};

use futures::channel::mpsc;

use std::fmt;

const EVENT_CHANNEL_CAPACITY: usize = 1000;

pub(crate) type EventSender = mpsc::Sender<Event>;
pub(crate) type EventReceiver = mpsc::Receiver<Event>;

pub(crate) fn channel() -> (EventSender, EventReceiver) {
    mpsc::channel(EVENT_CHANNEL_CAPACITY)
}

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
        endpoint: Endpoint,
        origin: Origin,
        sender: BytesSender,
    },

    LostConnection {
        epid: EndpointId,
    },

    EndpointConnected {
        epid: EndpointId,
        address: Address,
        origin: Origin,
        timestamp: u64,
        total: usize,
    },

    EndpointDisconnected {
        epid: EndpointId,
        total: usize,
    },

    MessageSent {
        epid: EndpointId,
        num_bytes: usize,
    },

    MessageReceived {
        epid: EndpointId,
        bytes: Vec<u8>,
    },

    TryConnect {
        epid: EndpointId,
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

            Event::NewConnection { endpoint, .. } => write!(f, "Event::NewConnection {{ {} }}", endpoint.id),

            Event::LostConnection { epid, .. } => write!(f, "Event::LostConnection {{ {} }}", epid),

            Event::EndpointConnected {
                epid,
                address,
                origin,
                timestamp,
                total,
            } => write!(
                f,
                "Event::EndpointConnected {{ {}, address: {}, origin: {}, ts: {}, num_connected: {} }}",
                epid, address, origin, timestamp, total
            ),

            Event::EndpointDisconnected { epid, total } => write!(
                f,
                "Event::EndpointDisconnected {{ {}, num_connected: {} }}",
                epid, total
            ),

            Event::MessageSent { epid, num_bytes } => {
                write!(f, "Event::MessageSent {{ {}, num_bytes: {} }}", epid, num_bytes)
            }

            Event::MessageReceived { epid, bytes } => {
                write!(f, "Event::MessageReceived {{ {}, num_bytes: {} }}", epid, bytes.len())
            }

            Event::TryConnect { epid, .. } => write!(f, "Event::TryConnect {{ {} }}", epid),
        }
    }
}
