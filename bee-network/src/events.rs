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
    endpoint::{connected::DataSender, Endpoint, EndpointId},
    tcp::connection::Origin,
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

    ConnectionCreated {
        endpoint: Endpoint,
        origin: Origin,
        data_sender: DataSender,
        timestamp: u64,
    },

    ConnectionDropped {
        epid: EndpointId,
    },

    EndpointConnected {
        epid: EndpointId,
        address: Address,
        origin: Origin,
        total: usize,
    },

    EndpointDisconnected {
        epid: EndpointId,
        total: usize,
    },

    MessageReceived {
        epid: EndpointId,
        message: Vec<u8>,
    },

    TryConnectTo {
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

            Event::ConnectionCreated { endpoint, .. } => write!(f, "Event::ConnectionCreated {{ {} }}", endpoint.epid),

            Event::ConnectionDropped { epid, .. } => write!(f, "Event::ConnectionDropped {{ {} }}", epid),

            Event::EndpointConnected {
                epid,
                address,
                origin,
                total,
            } => write!(
                f,
                "Event::EndpointConnected {{ {}, address: {}, origin: {}, num_connected: {} }}",
                epid, address, origin, total
            ),

            Event::EndpointDisconnected { epid, total } => write!(
                f,
                "Event::EndpointDisconnected {{ {}, num_connected: {} }}",
                epid, total
            ),

            Event::MessageReceived { epid, message } => {
                write!(f, "Event::MessageReceived {{ {}, num_bytes: {} }}", epid, message.len())
            }

            Event::TryConnectTo { epid, .. } => write!(f, "Event::TryConnectTo {{ {} }}", epid),
        }
    }
}
