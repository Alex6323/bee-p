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

use super::{
    connection::{MuxedConnection, Origin},
    errors::Error,
};

use crate::{
    interaction::events::InternalEventSender,
    peers::{BannedAddrList, BannedPeerList, PeerList},
    transport::build_transport,
    PeerId, PEER_LIMIT,
};

use log::*;

use libp2p::{identity, multiaddr::Protocol, Multiaddr, Transport};

use std::{net::IpAddr, sync::atomic::Ordering};

pub async fn dial(
    peer_address: Multiaddr,
    peer_id: Option<PeerId>,
    local_keys: &identity::Keypair,
    internal_event_sender: &InternalEventSender,
    peers: &PeerList,
    banned_addrs: &BannedAddrList,
    banned_peers: &BannedPeerList,
) -> Result<(), Error> {
    // Check, if we haven't yet reached the peer limit
    if peers.num_connected() >= PEER_LIMIT.load(Ordering::Relaxed) {
        warn!("Dialing aborted. Cause: Peer limit reached.");
        return Err(Error::PeerLimitReached(PEER_LIMIT.load(Ordering::Relaxed)));
    }

    let transport = build_transport(local_keys).map_err(|_| Error::CreatingTransportFailed)?;

    trace!("Dialing {} ({:?})...", peer_address, peer_id);

    let ip_address = match peer_address.iter().next().unwrap() {
        Protocol::Ip4(ip_addr) => IpAddr::V4(ip_addr),
        Protocol::Ip6(ip_addr) => IpAddr::V6(ip_addr),
        _ => return Err(Error::InvalidMultiaddr),
    };

    if banned_addrs.contains(&ip_address) {
        warn!("Dialing aborted. Cause: Banned IP {}.", ip_address);
        return Err(Error::DialedBannedAddress(ip_address.clone()));
    }

    let (id, muxer) = transport
        .dial(peer_address.clone())
        .map_err(|_| Error::DialingFailed(peer_address.clone()))?
        .await
        .map_err(|_| Error::DialingFailed(peer_address.clone()))?;

    if peer_id.is_some() && &id != peer_id.as_ref().unwrap() {
        warn!("Remote returned a different Peer Id than expected.");

        // Note: `peer_id.is_some() == true`
        return Err(Error::PeerIdMismatch {
            expected: peer_id.unwrap(),
            received: id,
        });
    }

    if banned_peers.contains(&id) {
        warn!("Tried to connect to a banned peer ({}).", id);
        return Err(Error::DialedBannedPeer(id));
    }

    if peers.contains_peer(&id) {
        debug!("Already connected to {}", id);
        return Err(Error::DuplicateConnection(id));
    }

    let connection = MuxedConnection::new(id, peer_address, muxer, Origin::Outbound);

    trace!(
        "Sucessfully created outbound connection to {} ({}).",
        connection.peer_address,
        connection.peer_id,
    );

    super::spawn_connection_handler(connection, internal_event_sender.clone()).await?;

    Ok(())
}
