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

use crate::{Multiaddr, PeerId};

use thiserror::Error as ErrorAttr;

use std::net::IpAddr;

#[derive(Debug, ErrorAttr)]
pub enum Error {
    #[error("Tried to connect to more peers than allowed ({}).", .0)]
    PeerLimitReached(usize),
    #[error("Building the underlying transport layer failed.")]
    CreatingTransportFailed,
    #[error("Binding to {} failed.", .0)]
    BindingAddressFailed(Multiaddr),
    #[error("Not listening on an address.")]
    NotListeningError,
    #[error("Failed to extract the IP address from a multiaddress.")]
    InvalidMultiaddr,
    #[error("Tried to dial a banned address: {}.", .0)]
    DialedBannedAddress(IpAddr),
    #[error("Tried to dial a banned peer: {}.", .0)]
    DialedBannedPeer(PeerId),
    #[error("Failed dialing address: {}.", .0)]
    DialingFailed(Multiaddr),
    #[error("Already connected to peer: {}.", .0)]
    DuplicateConnection(PeerId),
    #[error("Peer identifies with {}, but we expected: {}", .received, .expected)]
    PeerIdMismatch { expected: PeerId, received: PeerId },
    #[error("Creating outbound substream failed.")]
    CreatingOutboundSubstreamFailed,
    #[error("Creating inbound substream failed.")]
    CreatingInboundSubstreamFailed,
    #[error("Failed to upgrade a substream.")]
    SubstreamProtocolUpgradeFailed,
    #[error("Failed to send an internal event ({}).", .0)]
    InternalEventSendFailure(&'static str),
    #[error("Failed to send the message by writing to an underlying stream.")]
    MessageSendError,
    #[error("Failed to recv the message by reading from an underlying stream.")]
    MessageRecvError,
    #[error("The remote peer stopped the stream (EOF).")]
    StreamClosedByRemote,
}
