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

use crate::Multiaddr;

use thiserror::Error as ErrorAttr;

#[derive(Debug, ErrorAttr)]
pub enum Error {
    #[error("Building the underlying transport layer failed.")]
    CreatingTransportFailed,
    #[error("Binding to {} failed.", .0)]
    BindingAddressFailed(Multiaddr),
    #[error("Not listening on an address.")]
    NotListeningError,
    #[error("Tried to dial a banned address: {}.", .0)]
    DialedBannedAddress(Multiaddr),
    #[error("Tried to dial a banned peer: {}.", .0)]
    DialedBannedPeer(String),
    #[error("Tried to dial an unlisted peer: {}.", .0)]
    DialedUnlistedPeer(String),
    #[error("Failed dialing address: {}.", .0)]
    DialingFailed(Multiaddr),
    #[error("Already connected to peer: {}.", .0)]
    DuplicateConnection(String),
    #[error("Peer identifies with {}, but we expected: {}", .received, .expected)]
    PeerIdMismatch { expected: String, received: String },
    #[error("Creating outbound substream with {} failed.", .0)]
    CreatingOutboundSubstreamFailed(String),
    #[error("Creating inbound substream with {} failed.", .0)]
    CreatingInboundSubstreamFailed(String),
    #[error("Failed to upgrade a substream with {}.", .0)]
    SubstreamProtocolUpgradeFailed(String),
    #[error("Failed to send an internal event ({}).", .0)]
    InternalEventSendFailure(&'static str),
    #[error("Failed to write a message to a stream.")]
    MessageSendError,
    #[error("Failed to read a message from a stream.")]
    MessageRecvError,
    #[error("The remote peer {} stopped the stream (EOF).", 0)]
    StreamClosedByRemote,
}
