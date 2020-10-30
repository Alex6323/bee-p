use super::{
    connection::{Connection, Origin},
    Error,
};
use crate::event::EventSender;

use log::*;

use libp2p::{
    core::{muxing::StreamMuxerBox, upgrade},
    identity, noise, tcp, yamux, Multiaddr, PeerId, Transport,
};

use std::io;

pub async fn connect_peer(
    local_keys: identity::Keypair,
    remote_id: PeerId,
    remote_addr: Multiaddr,
    internal_event_sender: EventSender,
) -> Result<(), Error> {
    let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
        .into_authentic(&local_keys)
        .expect("error creating noise keys");

    let transport = tcp::TokioTcpConfig::default()
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
        .multiplex(yamux::Config::default())
        .map(|(peer, muxer), _| (peer, StreamMuxerBox::new(muxer)))
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        .boxed();

    trace!("Trying to connect to {} ({})...", remote_addr, remote_id);

    match transport.dial(remote_addr.clone()).unwrap().await {
        Ok((remote_id, stream)) => {
            let connection = match Connection::new(remote_id, remote_addr, stream, Origin::Outbound) {
                Ok(conn) => conn,
                Err(e) => {
                    warn!["Error creating connection: {:?}.", e];

                    return Err(Error::ConnectionAttemptFailed);
                }
            };

            trace!(
                "Sucessfully connected to {} ({}).",
                connection.remote_addr,
                connection.remote_id,
            );

            super::spawn_reader_writer(connection, internal_event_sender).await?;

            Ok(())
        }
        Err(e) => {
            warn!("Connecting to {} ({}) failed: {:?}.", remote_addr, remote_id, e);

            Err(Error::ConnectionAttemptFailed)
        }
    }
}
