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

use async_std::{
    net::{SocketAddr, ToSocketAddrs},
    task::block_on,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("Address could not be parsed.")]
    AddressParseError(#[from] std::io::Error),

    #[error("Address could not be resolved.")]
    AddressResolveError,
}

pub(crate) fn resolve_address(address: &str) -> Result<SocketAddr, Error> {
    block_on(address.to_socket_addrs())?
        .next()
        .map(|a| a.into())
        .ok_or(Error::AddressResolveError)
}
