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
    AddEndpoint { address: Multiaddr },
    RemoveEndpoint { address: Multiaddr },
    DialPeer { endpoint_address: Multiaddr },
    DisconnectPeer { id: PeerId },
    // TODO: maybe swap both fields, and rename `peer_id` to `to`?
    SendMessage { peer_id: PeerId, message: Vec<u8> },
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::AddEndpoint { address, .. } => write!(f, "Command::AddEndpoint {{ {} }}", address),
            Command::RemoveEndpoint { address, .. } => write!(f, "Command::RemoveEndpoint {{ {} }}", address),
            Command::DialPeer { endpoint_address, .. } => write!(f, "Command::DialPeer {{ {} }}", endpoint_address),
            Command::DisconnectPeer { id, .. } => write!(f, "Command::DisconnectPeer {{ {} }}", id),
            Command::SendMessage { peer_id, .. } => write!(f, "Command::SendMessage {{ {} }}", peer_id),
        }
    }
}
