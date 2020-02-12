use crate::messages::errors::MessageError;
use crate::messages::handshake::Handshake;
use crate::messages::heartbeat::Heartbeat;
use crate::messages::legacy_gossip::LegacyGossip;
use crate::messages::milestone_request::MilestoneRequest;
use crate::messages::transaction_broadcast::TransactionBroadcast;
use crate::messages::transaction_request::TransactionRequest;
use crate::messages::{Message, MessageType};

pub fn create_message(bytes: &[u8]) -> Result<MessageType, MessageError> {
    let header = Header::from_bytes(&bytes[Header::size_range()]);

    match bytes[0] {
        0x01 => Ok(MessageType::Handshake(Handshake::from_bytes(&bytes[3..])?)),
        0x02 => Ok(MessageType::LegacyGossip(LegacyGossip::from_bytes(
            &bytes[3..],
        )?)),
        0x03 => Ok(MessageType::MilestoneRequest(MilestoneRequest::from_bytes(
            &bytes[3..],
        )?)),
        0x04 => Ok(MessageType::TransactionBroadcast(
            TransactionBroadcast::from_bytes(&bytes[3..])?,
        )),
        0x05 => Ok(MessageType::TransactionRequest(
            TransactionRequest::from_bytes(&bytes[3..])?,
        )),
        0x06 => Ok(MessageType::Heartbeat(Heartbeat::from_bytes(&bytes[3..])?)),
        _ => Err(MessageError::UnknownMessageType(bytes[0])),
    }
}
