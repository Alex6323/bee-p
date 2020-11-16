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

use std::net::IpAddr;

pub type CommandSender = flume::Sender<Command>;
pub type CommandReceiver = flume::Receiver<Command>;

pub fn channel() -> (CommandSender, CommandReceiver) {
    flume::unbounded()
}

#[derive(Debug)]
pub enum Command {
    ConnectPeer { address: Multiaddr, id: PeerId },
    ConnectUnknownPeer { address: Multiaddr },
    DisconnectPeer { id: PeerId },
    SendMessage { message: Vec<u8>, to: PeerId },
    BanAddr { ip: IpAddr },
    BanPeer { id: PeerId },
    UnbanAddr { ip: IpAddr },
    UnbanPeer { id: PeerId },
}
