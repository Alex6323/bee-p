mod errors;
mod handshake;
mod header;
mod heartbeat;
mod legacy_gossip;
mod message;
mod message_builder;
mod milestone_request;
mod transaction_broadcast;
mod transaction_request;

pub use errors::MessageError;
pub use handshake::Handshake;
pub use header::Header;
pub use heartbeat::Heartbeat;
pub use legacy_gossip::LegacyGossip;
pub use message::Message;
pub use milestone_request::MilestoneRequest;
pub use transaction_broadcast::TransactionBroadcast;
pub use transaction_request::TransactionRequest;

use std::convert::TryFrom;

pub enum MessageType {
    Header,
    Handshake,
    LegacyGossip,
    MilestoneRequest,
    TransactionBroadcast,
    TransactionRequest,
    Heartbeat,
}

impl TryFrom<u8> for MessageType {
    type Error = MessageError;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        return match byte {
            0x00 => Ok(MessageType::Header),
            0x01 => Ok(MessageType::Handshake),
            0x02 => Ok(MessageType::LegacyGossip),
            0x03 => Ok(MessageType::MilestoneRequest),
            0x04 => Ok(MessageType::TransactionBroadcast),
            0x05 => Ok(MessageType::TransactionRequest),
            0x06 => Ok(MessageType::Heartbeat),
            _ => Err(MessageError::UnknownMessageType(byte)),
        };
    }
}
