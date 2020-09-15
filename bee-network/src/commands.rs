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

use futures::channel::mpsc;

use std::fmt;

pub fn channel() -> (mpsc::UnboundedSender<Command>, mpsc::UnboundedReceiver<Command>) {
    mpsc::unbounded()
}

#[derive(Debug)]
pub enum Command {
    AddContact { url: String },
    RemoveContact { url: String },
    ConnectEndpoint { epid: EndpointId },
    DisconnectEndpoint { epid: EndpointId },
    SendMessage { epid: EndpointId, message: Vec<u8> },
    SetDuplicate { epid: EndpointId, of: EndpointId },
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::AddContact { url, .. } => write!(f, "Command::AddContact {{ {} }}", url),
            Command::RemoveContact { url, .. } => write!(f, "Command::RemoveContact {{ {} }}", url),
            Command::ConnectEndpoint { epid, .. } => write!(f, "Command::ConnectEndpoint {{ {} }}", epid),
            Command::DisconnectEndpoint { epid, .. } => write!(f, "Command::DisconnectEndpoint {{ {} }}", epid),
            Command::SendMessage { epid, .. } => write!(f, "Command::SendMessage {{ {} }}", epid),
            Command::SetDuplicate { epid, of } => write!(f, "Command::SetDuplicate {{ {} == {} }}", epid, of),
        }
    }
}
