//! Messages of the protocol version 1

#[allow(dead_code)]
mod legacy_gossip;

/// Version identifier of the messages version 1
#[allow(dead_code)]
pub(crate) const MESSAGES_VERSION_1: u8 = 1 << 0;

#[allow(unused_imports)]
pub(crate) use legacy_gossip::LegacyGossip;
