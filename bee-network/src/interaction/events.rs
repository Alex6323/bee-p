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
    conns::Origin,
    peers::{DataSender, PeerInfo},
    Multiaddr, PeerId,
};

use super::commands::Command;

pub type EventReceiver = flume::Receiver<Event>;
pub type EventSender = flume::Sender<Event>;
pub type InternalEventReceiver = flume::Receiver<InternalEvent>;
pub type InternalEventSender = flume::Sender<InternalEvent>;

pub fn channel<T>() -> (flume::Sender<T>, flume::Receiver<T>) {
    flume::unbounded()
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Event {
    PeerAdded { id: PeerId },
    PeerRemoved { id: PeerId },
    PeerConnected { id: PeerId, address: Multiaddr },
    PeerDisconnected { id: PeerId },
    MessageReceived { message: Vec<u8>, from: PeerId },
    PeerBanned { id: PeerId },
    AddressBanned { address: Multiaddr },
    CommandFailed { command: Command }, // TODO: maybe we should provide the reason as well!
}

#[derive(Debug)]
pub enum InternalEvent {
    ConnectionEstablished {
        peer_id: PeerId,
        peer_info: PeerInfo,
        origin: Origin,
        message_sender: DataSender,
    },
    ConnectionDropped {
        peer_id: PeerId,
    },
    MessageReceived {
        message: Vec<u8>,
        from: PeerId,
    },
    ReconnectScheduled {
        peer_id: PeerId,
    },
}
