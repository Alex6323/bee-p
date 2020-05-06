//! Messages of the protocol version 0

mod handshake;

/// Version identifier of the messages version 0
pub(crate) const MESSAGES_VERSION_0: u8 = 0;

pub(crate) use handshake::Handshake;
