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

use std::net::IpAddr;

pub type EventReceiver = flume::Receiver<Event>;
pub type EventSender = flume::Sender<Event>;
pub type InternalEventReceiver = flume::Receiver<InternalEvent>;
pub type InternalEventSender = flume::Sender<InternalEvent>;

pub fn channel<T>() -> (flume::Sender<T>, flume::Receiver<T>) {
    flume::unbounded()
}

#[derive(Debug)]
pub enum InternalEvent {
    ConnectionEstablished {
        peer_id: PeerId,
        peer_address: Multiaddr,
        origin: Origin,
        message_sender: DataSender,
    },
    ConnectionDropped {
        peer_id: PeerId,
        peer_address: Multiaddr,
    },
    ReconnectScheduled {
        peer_id: PeerId,
        peer_address: Multiaddr,
    },
    MessageReceived {
        message: Vec<u8>,
        sender: PeerId,
    },
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Event {
    PeerConnected {
        id: PeerId,
        address: Multiaddr,
        origin: Origin,
    },

    PeerDisconnected {
        id: PeerId,
    },

    MessageReceived {
        message: Vec<u8>,
        sender: PeerId,
    },
    PeerBanned {
        id: PeerId,
    },
    IpBanned {
        ip: IpAddr,
    },
}
