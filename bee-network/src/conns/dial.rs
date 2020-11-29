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

use super::{errors::Error, Origin};

use crate::{
    interaction::events::InternalEventSender,
    peers::{BannedAddrList, BannedPeerList, PeerInfo, PeerList, PeerRelation},
    transport::build_transport,
    Multiaddr, PeerId, ShortId,
};

use log::*;

use libp2p::{identity, Transport};

pub async fn dial_peer(
    peer_id: PeerId,
    local_keys: &identity::Keypair,
    internal_event_sender: &InternalEventSender,
    peers: &PeerList,
    banned_addrs: &BannedAddrList,
    banned_peers: &BannedPeerList,
) -> Result<(), Error> {
    // Prevent duplicate connections.
    if peers.is_connected(&peer_id) {
        return Err(Error::DuplicateConnection(peer_id.short()));
    }

    // Prevent dialing banned peers.
    if banned_peers.contains(&peer_id) {
        return Err(Error::DialedBannedPeer(peer_id.short()));
    }

    // Prevent dialing unlisted/unregistered peers.
    let peer_info = peers
        .get_info(&peer_id)
        .ok_or(Error::DialedUnlistedPeer(peer_id.short()))?;

    // Prevent dialing banned addresses.
    if banned_addrs.contains(&peer_info.address.to_string()) {
        return Err(Error::DialedBannedAddress(peer_info.address));
    }

    log_dialing_peer(&peer_id, &peer_info);

    let (id, muxer) = build_transport(local_keys)
        .map_err(|_| Error::CreatingTransportFailed)?
        .dial(peer_info.address.clone())
        .map_err(|_| Error::DialingFailed(peer_info.address.clone()))?
        .await
        .map_err(|_| Error::DialingFailed(peer_info.address.clone()))?;

    // Prevent connecting to dishonest peers or peers we have no up-to-date information about.
    if id != peer_id {
        return Err(Error::PeerIdMismatch {
            expected: peer_id.to_string(),
            received: id.to_string(),
        });
    }

    let peer_id = id;

    log_outbound_connection_success(&peer_id, &peer_info);

    super::spawn_connection_handler(
        peer_id,
        peer_info,
        muxer,
        Origin::Outbound,
        internal_event_sender.clone(),
    )
    .await?;

    Ok(())
}

#[inline]
fn log_dialing_peer(peer_id: &PeerId, peer_info: &PeerInfo) {
    if let Some(alias) = peer_info.alias.as_ref() {
        info!("Dialing '{}/{}' [{}]...", alias, peer_id.short(), peer_info.address,);
    } else {
        info!("Dialing '{}' [{}]...", peer_id.short(), peer_info.address);
    }
}

pub async fn dial_address(
    address: Multiaddr,
    local_keys: &identity::Keypair,
    internal_event_sender: &InternalEventSender,
    peers: &PeerList,
    banned_addrs: &BannedAddrList,
    banned_peers: &BannedPeerList,
) -> Result<(), Error> {
    // Prevent dialing banned addresses.
    if banned_addrs.contains(&address.to_string()) {
        return Err(Error::DialedBannedAddress(address));
    }

    info!("Dialing [{}]...", address);

    let (peer_id, muxer) = build_transport(local_keys)
        .map_err(|_| Error::CreatingTransportFailed)?
        .dial(address.clone())
        .map_err(|_| Error::DialingFailed(address.clone()))?
        .await
        .map_err(|_| Error::DialingFailed(address.clone()))?;

    // Prevent duplicate connections.
    if peers.is_connected(&peer_id) {
        return Err(Error::DuplicateConnection(peer_id.short()));
    }

    // Prevent dialing banned peers.
    if banned_peers.contains(&peer_id) {
        return Err(Error::DialedBannedPeer(peer_id.short()));
    }

    let peer_info = PeerInfo {
        address,
        alias: None,
        relation: PeerRelation::Unknown,
    };

    log_outbound_connection_success(&peer_id, &peer_info);

    super::spawn_connection_handler(
        peer_id,
        peer_info,
        muxer,
        Origin::Outbound,
        internal_event_sender.clone(),
    )
    .await?;

    Ok(())
}

#[inline]
fn log_outbound_connection_success(peer_id: &PeerId, peer_info: &PeerInfo) {
    if let Some(alias) = peer_info.alias.as_ref() {
        info!(
            "Established (outbound) connection with '{}/{}' [{}].",
            alias,
            peer_id.short(),
            peer_info.address,
        )
    } else {
        info!(
            "Established (outbound) connection with '{}' [{:?}].",
            peer_id.short(),
            peer_info.address,
        );
    }
}
