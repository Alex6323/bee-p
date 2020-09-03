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
use crate::utils::time;

use async_std::{
    net::{SocketAddr, TcpStream},
    sync::Arc,
};

use std::fmt;

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

#[derive(Clone)]
pub struct Connection {
    pub origin: Origin,
    pub local_addr: SocketAddr,
    pub remote_addr: SocketAddr,
    pub stream: Arc<TcpStream>,
    pub timestamp: u64,
}

impl Connection {
    pub fn new(stream: TcpStream, origin: Origin) -> Result<Self, Error> {
        let local_addr = stream.local_addr()?;
        let remote_addr = stream.peer_addr()?;
        let stream = Arc::new(stream);

        Ok(Self {
            origin,
            local_addr,
            remote_addr,
            stream,
            timestamp: time::timestamp_millis(),
        })
    }
}

impl fmt::Display for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} <-> {}", self.local_addr, self.remote_addr)
    }
}

impl Eq for Connection {}
impl PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.remote_addr == other.remote_addr
    }
}
