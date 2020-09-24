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

use thiserror::Error;
use tokio::net;

use std::{fmt, net::SocketAddr};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Address could not be parsed.")]
    AddressParseError(#[from] std::io::Error),

    #[error("Address could not be resolved.")]
    AddressResolveError,
}

pub async fn resolve_address(address: &str) -> Result<SocketAddr, Error> {
    net::lookup_host(address)
        .await?
        .next()
        .ok_or(Error::AddressResolveError)
}

pub type Port = u16;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum TransportProtocol {
    Tcp,
    Udp,
}

impl TransportProtocol {
    pub fn is_tcp(self) -> bool {
        self == TransportProtocol::Tcp
    }

    pub fn is_udp(self) -> bool {
        self == TransportProtocol::Udp
    }
}

impl fmt::Display for TransportProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let protocol = match *self {
            TransportProtocol::Tcp => "tcp",
            TransportProtocol::Udp => "udp",
        };

        write!(f, "{}", protocol)
    }
}
