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

use crate::endpoint::EndpointId;

use std::fmt;

pub type CommandSender = flume::Sender<Command>;
pub type CommandReceiver = flume::Receiver<Command>;

pub fn channel() -> (CommandSender, CommandReceiver) {
    flume::unbounded()
}

#[derive(Debug)]
pub enum Command {
    AddEndpoint {
        url: String,
    },
    RemoveEndpoint {
        epid: EndpointId,
    },
    ConnectEndpoint {
        epid: EndpointId,
    },
    DisconnectEndpoint {
        epid: EndpointId,
    },
    SendMessage {
        receiver_epid: EndpointId,
        message: Vec<u8>,
    },
    MarkDuplicate {
        duplicate_epid: EndpointId,
        original_epid: EndpointId,
    },
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::AddEndpoint { url, .. } => write!(f, "Command::AddEndpoint {{ {} }}", url),
            Command::RemoveEndpoint { epid, .. } => write!(f, "Command::RemoveEndpoint {{ {} }}", epid),
            Command::ConnectEndpoint { epid, .. } => write!(f, "Command::ConnectEndpoint {{ {} }}", epid),
            Command::DisconnectEndpoint { epid, .. } => write!(f, "Command::DisconnectEndpoint {{ {} }}", epid),
            Command::SendMessage { receiver_epid, .. } => write!(f, "Command::SendMessage {{ {} }}", receiver_epid),
            Command::MarkDuplicate {
                duplicate_epid,
                original_epid,
            } => write!(
                f,
                "Command::MarkDuplicate {{ {} == {} }}",
                duplicate_epid, original_epid
            ),
        }
    }
}
