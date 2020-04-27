use crate::{
    config::slice_eq,
    message::Handshake,
    peer::Peer,
    protocol::{
        supported_version,
        Protocol,
    },
};

use bee_network::{
    Origin,
    Port,
};

use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

pub(crate) enum HandshakeError {
    InvalidTimestampDiff(i64),
    CoordinatorMismatch,
    MwmMismatch(u8, u8),
    UnsupportedVersion(u8),
    PortMismatch(u16, u16),
    UnboundPeer,
}

pub(crate) fn validate_handshake(peer: &Peer, handshake: Handshake) -> Result<(), HandshakeError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Clock may have gone backwards")
        .as_millis() as u64;

    if ((timestamp - handshake.timestamp) as i64).abs() > 5000 {
        Err(HandshakeError::InvalidTimestampDiff(
            ((timestamp - handshake.timestamp) as i64).abs(),
        ))?
    }

    if !slice_eq(
        &Protocol::get().config.coordinator.public_key_bytes,
        &handshake.coordinator,
    ) {
        Err(HandshakeError::CoordinatorMismatch)?
    }

    if Protocol::get().config.mwm != handshake.minimum_weight_magnitude {
        Err(HandshakeError::MwmMismatch(
            Protocol::get().config.mwm,
            handshake.minimum_weight_magnitude,
        ))?
    }

    if let Err(version) = supported_version(&handshake.supported_versions) {
        Err(HandshakeError::UnsupportedVersion(version))?
    }

    match peer.origin {
        Origin::Outbound => {
            if peer.address.port() != Port(handshake.port) {
                Err(HandshakeError::PortMismatch(*peer.address.port(), handshake.port))?
            }
        }
        Origin::Inbound => {
            // TODO check if whitelisted
        }
        Origin::Unbound => Err(HandshakeError::UnboundPeer)?,
    }

    Ok(())
}
