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

use crate::{address::url::Url, endpoint::EndpointId};

use futures::channel::mpsc;

use std::fmt;

const COMMAND_CHANNEL_CAPACITY: usize = 1000;

pub(crate) fn channel() -> (mpsc::Sender<Command>, mpsc::Receiver<Command>) {
    mpsc::channel(COMMAND_CHANNEL_CAPACITY)
}

#[derive(Debug)]
pub enum Command {
    AddEndpoint { url: Url },
    RemoveEndpoint { epid: EndpointId },
    Connect { epid: EndpointId },
    Disconnect { epid: EndpointId },
    SendMessage { epid: EndpointId, message: Vec<u8> },
    SetDuplicate { epid: EndpointId, other: EndpointId },
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::AddEndpoint { url, .. } => write!(f, "Command::AddEndpoint {{ {} }}", url),
            Command::RemoveEndpoint { epid, .. } => write!(f, "Command::RemoveEndpoint {{ {} }}", epid),
            Command::Connect { epid, .. } => write!(f, "Command::Connect {{ {} }}", epid),
            Command::Disconnect { epid, .. } => write!(f, "Command::Disconnect {{ {} }}", epid),
            Command::SendMessage { epid, .. } => write!(f, "Command::SendMessage {{ {} }}", epid),
            Command::SetDuplicate { epid, other } => write!(f, "Command::SetDuplicate {{ {} == {} }}", epid, other),
        }
    }
}
