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

use bee_network::{EndpointId, Origin};

use std::net::SocketAddr;

pub(crate) struct Peer {
    pub(crate) epid: EndpointId,
    pub(crate) address: SocketAddr,
    pub(crate) origin: Origin,
}

impl Peer {
    pub fn new(epid: EndpointId, address: SocketAddr, origin: Origin) -> Self {
        Self { epid, address, origin }
    }
}
