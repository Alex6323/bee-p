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

use crate::{conns::Error as ConnError, Multiaddr};

use thiserror::Error as ErrorAttr;

#[derive(Debug, ErrorAttr)]
pub enum Error {
    #[error("Failed to send an event ({}).", .0)]
    EventSendFailure(&'static str),
    #[error("Failed to send an internal event ({}).", .0)]
    InternalEventSendFailure(&'static str),
    #[error("Failed to send a message to {}", .0)]
    SendMessageFailure(String),
    #[error("Unlisted peer: {}", .0)]
    UnlistedPeer(String),
    #[error("Disconnected peer: {}", .0)]
    DisconnectedPeer(String),
    #[error("Failed to connect to peer. Cause: {:?}", .0)]
    ConnectFailure(ConnError),
    #[error("Failed to disconnect from peer: {}", .0)]
    DisconnectPeerFailure(String),
    #[error("Already banned that address: {}", .0)]
    AddressAlreadyBanned(Multiaddr),
    #[error("Already banned that peer: {}", .0)]
    PeerAlreadyBanned(String),
    #[error("Already unbanned that address: {}", .0)]
    AddressAlreadyUnbanned(Multiaddr),
    #[error("Already unbanned that peer: {}", .0)]
    PeerAlreadyUnbanned(String),
    #[error("Already added that peer: {}", .0)]
    PeerAlreadyAdded(String),
    #[error("Already removed that peer: {}", .0)]
    PeerAlreadyRemoved(String),
    #[error("Tried to add more known peers than allowed ({}).", .0)]
    KnownPeerLimitReached(usize),
    #[error("Tried to add more unknown peers than allowed ({}).", .0)]
    UnknownPeerLimitReached(usize),
}
