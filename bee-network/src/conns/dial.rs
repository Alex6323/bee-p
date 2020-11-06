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
    Error,
};
use crate::{interaction::events::EventSender, peers::ConnectedPeerList, transport::build_transport};

use log::*;

use libp2p::{identity, Multiaddr, Transport};

pub async fn dial_peer(
    endpoint_address: Multiaddr,
    local_keys: &identity::Keypair,
    internal_event_sender: EventSender,
    connected_peers: &ConnectedPeerList,
) -> Result<(), Error> {
    let transport = build_transport(local_keys)?;

    trace!("Dialing {}...", endpoint_address);

    // TODO: error handling
    match transport.dial(endpoint_address.clone()).expect("dial").await {
        Ok((peer_id, muxer)) => {
            if !connected_peers.contains(&peer_id) {
                let connection = match MuxedConnection::new(peer_id, endpoint_address, muxer, Origin::Outbound) {
                    Ok(conn) => conn,
                    Err(e) => {
                        warn!["Error creating multiplexed connection: {:?}.", e];

                        return Err(Error::ConnectionAttemptFailed);
                    }
                };

                trace!(
                    "Sucessfully connected to {} ({}).",
                    connection.endpoint_address,
                    connection.peer_id,
                );

                super::spawn_connection_handler(connection, internal_event_sender).await?;
            } else {
                trace!("Already connected to {}", peer_id);
            }

            Ok(())
        }
        Err(e) => {
            warn!("Dialing {} failed: {:?}.", endpoint_address, e);

            Err(Error::ConnectionAttemptFailed)
        }
    }
}
