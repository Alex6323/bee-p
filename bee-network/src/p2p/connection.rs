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

use super::Error;

use libp2p::{core::muxing::StreamMuxerBox, Multiaddr, PeerId};

use std::fmt;

// #[derive(Clone)]
pub struct Connection {
    pub remote_id: PeerId,
    pub remote_addr: Multiaddr,
    pub stream: StreamMuxerBox,
    pub origin: Origin,
}

impl Connection {
    pub fn new(
        remote_id: PeerId,
        remote_addr: Multiaddr,
        stream: StreamMuxerBox,
        origin: Origin,
    ) -> Result<Self, Error> {
        Ok(Self {
            remote_id,
            remote_addr,
            stream,
            origin,
        })
    }
}

impl Eq for Connection {}
impl PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.remote_id == other.remote_id
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Origin {
    Inbound,
    Outbound,
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match *self {
            Origin::Outbound => "outbound",
            Origin::Inbound => "inbound",
        };
        write!(f, "{}", s)
    }
}
