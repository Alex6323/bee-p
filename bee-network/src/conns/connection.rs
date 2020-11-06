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

pub struct MuxedConnection {
    pub peer_id: PeerId,
    pub endpoint_address: Multiaddr,
    pub muxer: StreamMuxerBox,
    pub origin: Origin,
}

impl MuxedConnection {
    pub fn new(
        peer_id: PeerId,
        endpoint_address: Multiaddr,
        muxer: StreamMuxerBox,
        origin: Origin,
    ) -> Result<Self, Error> {
        Ok(Self {
            peer_id,
            endpoint_address,
            muxer,
            origin,
        })
    }
}

impl Eq for MuxedConnection {}
impl PartialEq for MuxedConnection {
    fn eq(&self, other: &Self) -> bool {
        self.peer_id == other.peer_id
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
