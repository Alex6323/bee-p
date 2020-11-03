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

use crate::{conns::Origin, peers::DataSender};

use libp2p::{Multiaddr, PeerId};
use std::fmt;

pub type EventSender = flume::Sender<Event>;
pub type EventReceiver = flume::Receiver<Event>;

pub fn channel() -> (EventSender, EventReceiver) {
    flume::unbounded()
}

// TODO: add InternalEvent for the Connection... events

#[derive(Debug)]
#[non_exhaustive]
pub enum Event {
    PeerAdded {
        peer_address: Multiaddr,
    },

    PeerRemoved {
        peer_address: Multiaddr,
        peer_id: Option<PeerId>,
    },

    ConnectionEstablished {
        peer_address: Multiaddr,
        peer_id: PeerId,
        origin: Origin,
        data_sender: DataSender,
    },

    ConnectionDropped {
        peer_address: Multiaddr,
        peer_id: PeerId,
    },

    PeerConnected {
        peer_address: Multiaddr,
        peer_id: PeerId,
        origin: Origin,
    },

    PeerDisconnected {
        peer_id: PeerId,
    },

    MessageReceived {
        peer_id: PeerId,
        message: Vec<u8>,
    },

    ReconnectTimerElapsed {
        peer_address: Multiaddr,
    },
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::PeerAdded { peer_address } => write!(f, "Event::PeerAdded {{ {} }}", peer_address),

            Event::PeerRemoved { peer_address, .. } => write!(f, "Event::PeerRemoved {{ {} }}", peer_address),

            Event::ConnectionEstablished {
                peer_id, peer_address, ..
            } => write!(f, "Event::ConnectionEstablished {{ {} ({}) }}", peer_address, peer_id),

            Event::ConnectionDropped {
                peer_id, peer_address, ..
            } => write!(f, "Event::ConnectionDropped {{ {} ({}) }}", peer_address, peer_id),

            Event::PeerConnected {
                peer_id,
                peer_address,
                origin,
            } => write!(
                f,
                "Event::PeerConnected {{ {}, peer_address: {}, origin: {} }}",
                peer_id, peer_address, origin
            ),

            Event::PeerDisconnected { peer_id } => write!(f, "Event::PeerDisconnected {{ {} }}", peer_id),

            Event::MessageReceived { peer_id, message } => {
                write!(f, "Event::MessageReceived {{ {}, length: {} }}", peer_id, message.len())
            }

            Event::ReconnectTimerElapsed { peer_address, .. } => {
                write!(f, "Event::ReconnectTimerElapsed {{ {} }}", peer_address)
            }
        }
    }
}
