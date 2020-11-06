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

use libp2p::{Multiaddr, PeerId};

use std::fmt;

pub type CommandSender = flume::Sender<Command>;
pub type CommandReceiver = flume::Receiver<Command>;

pub fn channel() -> (CommandSender, CommandReceiver) {
    flume::unbounded()
}

#[derive(Debug)]
pub enum Command {
    AddPeer { peer_address: Multiaddr },
    RemovePeer { peer_address: Multiaddr },
    ConnectPeer { peer_address: Multiaddr },
    DisconnectPeer { peer_id: PeerId },
    SendMessage { peer_id: PeerId, message: Vec<u8> },
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::AddPeer { peer_address, .. } => write!(f, "Command::AddPeer {{ {} }}", peer_address),
            Command::RemovePeer { peer_address, .. } => write!(f, "Command::RemovePeer {{ {} }}", peer_address),
            Command::ConnectPeer { peer_address, .. } => write!(f, "Command::ConnectPeer {{ {} }}", peer_address),
            Command::DisconnectPeer { peer_id, .. } => write!(f, "Command::DisconnectPeer {{ {} }}", peer_id),
            Command::SendMessage { peer_id, .. } => write!(f, "Command::SendMessage {{ {} }}", peer_id),
        }
    }
}
