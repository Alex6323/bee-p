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

mod connected;
mod manager;
mod peerlist;

pub use connected::*;
pub use manager::*;
pub use peerlist::*;

use thiserror::Error;

/// Errors that can happen when dealing with `Address`es.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Error resolving domain name to address.")]
    Io(#[from] std::io::Error),

    #[error("Error parsing url.")]
    UrlParseFailure,

    #[error("Unspecified transport protocol.")]
    UnspecifiedTransportProtocol,

    #[error("Unsupported transport protocol.")]
    UnsupportedTransportProtocol,

    // TODO: rename to to 'DomainNameResolutionFailure'
    #[error("Error resolving domain name to address.")]
    DnsFailure,
}

pub type DataSender = flume::Sender<Vec<u8>>;
pub type DataReceiver = flume::Receiver<Vec<u8>>;

pub fn channel() -> (DataSender, DataReceiver) {
    flume::unbounded()
}
