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

use crate::peers::PeerRelation;

use libp2p::{Multiaddr, PeerId};

pub type CommandSender = flume::Sender<Command>;
pub type CommandReceiver = flume::Receiver<Command>;

pub fn channel() -> (CommandSender, CommandReceiver) {
    flume::unbounded()
}

#[derive(Debug, Eq, PartialEq)]
pub enum Command {
    AddPeer {
        id: PeerId,
        address: Multiaddr,
        alias: Option<String>,
        relation: PeerRelation,
    },
    RemovePeer {
        id: PeerId,
    },
    ConnectPeer {
        id: PeerId,
    },
    DisconnectPeer {
        id: PeerId,
    },
    DialAddress {
        address: Multiaddr,
    },
    SendMessage {
        message: Vec<u8>,
        to: PeerId,
    },
    BanAddress {
        address: Multiaddr,
    },
    BanPeer {
        id: PeerId,
    },
    UnbanAddress {
        address: Multiaddr,
    },
    UnbanPeer {
        id: PeerId,
    },
    UpdateRelation {
        id: PeerId,
        relation: PeerRelation,
    },
}
