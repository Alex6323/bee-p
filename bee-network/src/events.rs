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
    endpoint::{DataSender, EndpointId},
    tcp::connection::Origin,
};

use futures::{channel::mpsc, stream};

use std::{fmt, net::SocketAddr};

pub type EventSender = mpsc::UnboundedSender<Event>;
pub type EventReceiver = mpsc::UnboundedReceiver<Event>;

pub fn channel() -> (EventSender, EventReceiver) {
    mpsc::unbounded()
}

pub type Events = stream::Fuse<EventReceiver>;

#[derive(Debug)]
#[non_exhaustive]
pub enum Event {
    EndpointAdded {
        epid: EndpointId,
    },

    EndpointRemoved {
        epid: EndpointId,
    },

    ConnectionEstablished {
        epid: EndpointId,
        socket_address: SocketAddr,
        origin: Origin,
        sender: DataSender,
    },

    ConnectionDropped {
        epid: EndpointId,
    },

    EndpointConnected {
        epid: EndpointId,
        socket_address: SocketAddr,
        origin: Origin,
    },

    EndpointDisconnected {
        epid: EndpointId,
    },

    MessageReceived {
        epid: EndpointId,
        message: Vec<u8>,
    },

    TimerElapsed {
        epid: EndpointId,
    },
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::EndpointAdded { epid } => write!(f, "Event::EndpointAdded {{ {} }}", epid),

            Event::EndpointRemoved { epid } => write!(f, "Event::EndpointRemoved {{ {} }}", epid),

            Event::ConnectionEstablished { epid, .. } => write!(f, "Event::ConnectionEstablished {{ {} }}", epid),

            Event::ConnectionDropped { epid, .. } => write!(f, "Event::ConnectionDropped {{ {} }}", epid),

            Event::EndpointConnected {
                epid,
                socket_address,
                origin,
            } => write!(
                f,
                "Event::EndpointConnected {{ {}, socket_address: {}, origin: {} }}",
                epid, socket_address, origin
            ),

            Event::EndpointDisconnected { epid } => write!(f, "Event::EndpointDisconnected {{ {} }}", epid),

            Event::MessageReceived { epid, message } => {
                write!(f, "Event::MessageReceived {{ {}, num_bytes: {} }}", epid, message.len())
            }

            Event::TimerElapsed { epid, .. } => write!(f, "Event::TimerElapsed {{ {} }}", epid),
        }
    }
}
