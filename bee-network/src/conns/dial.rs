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
    connection::{Connection, Origin},
    Error,
};
use crate::event::EventSender;

use log::*;

use libp2p::{
    core::{muxing::StreamMuxerBox, upgrade},
    identity, noise, tcp, yamux, Multiaddr, Transport,
};

use std::io;

pub async fn dial_peer(
    local_keys: &identity::Keypair,
    peer_address: Multiaddr,
    internal_event_sender: EventSender,
) -> Result<(), Error> {
    let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
        .into_authentic(local_keys)
        .expect("error creating noise keys");

    // TODO: use build_tcp_ws_....

    let transport = tcp::TokioTcpConfig::default()
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
        .multiplex(yamux::Config::default())
        .map(|(peer, muxer), _| (peer, StreamMuxerBox::new(muxer)))
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        .boxed();

    trace!("Trying to connect to {}...", peer_address);

    match transport.dial(peer_address.clone()).unwrap().await {
        Ok((remote_id, stream)) => {
            let connection = match Connection::new(remote_id, peer_address, stream, Origin::Outbound) {
                Ok(conn) => conn,
                Err(e) => {
                    warn!["Error creating connection: {:?}.", e];

                    return Err(Error::ConnectionAttemptFailed);
                }
            };

            trace!(
                "Sucessfully connected to {} ({}).",
                connection.peer_address,
                connection.peer_id,
            );

            super::spawn_reader_writer(connection, internal_event_sender).await?;

            Ok(())
        }
        Err(e) => {
            warn!("Connecting to {} failed: {:?}.", peer_address, e);

            Err(Error::ConnectionAttemptFailed)
        }
    }
}
