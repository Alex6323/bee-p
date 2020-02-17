use crate::messages::errors::MessageError;
use crate::messages::handshake::Handshake;
use crate::messages::heartbeat::Heartbeat;
use crate::messages::legacy_gossip::LegacyGossip;
use crate::messages::milestone_request::MilestoneRequest;
use crate::messages::transaction_broadcast::TransactionBroadcast;
use crate::messages::transaction_request::TransactionRequest;
use crate::messages::{Message, MessageType};

use std::convert::TryInto;

pub fn read_message(bytes: &[u8]) -> Result<MessageType, MessageError> {
    if bytes.len() < 3 {
        Err(MessageError::InvalidHeaderLength(bytes.len()))?;
    }

    let message_type = bytes[0];
    // Safe to unwrap since we made sure it has the right size
    let message_length = u16::from_be_bytes(bytes[1..3].try_into().unwrap());
    let message = &bytes[3..];

    if message_length as usize != message.len() {
        Err(MessageError::InvalidAdvertisedMessageLength(
            message_length as usize,
            message.len(),
        ))?;
    }

    match message_type {
        0x01 => Ok(MessageType::Handshake(Handshake::from_bytes(message)?)),
        0x02 => Ok(MessageType::LegacyGossip(LegacyGossip::from_bytes(
            message,
        )?)),
        0x03 => Ok(MessageType::MilestoneRequest(MilestoneRequest::from_bytes(
            message,
        )?)),
        0x04 => Ok(MessageType::TransactionBroadcast(
            TransactionBroadcast::from_bytes(message)?,
        )),
        0x05 => Ok(MessageType::TransactionRequest(
            TransactionRequest::from_bytes(message)?,
        )),
        0x06 => Ok(MessageType::Heartbeat(Heartbeat::from_bytes(message)?)),
        _ => Err(MessageError::UnknownMessageType(message_type)),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn read_message_invalid_header_length_test() {
        match read_message(&[]) {
            Err(MessageError::InvalidHeaderLength(length)) => assert_eq!(length, 0),
            _ => unreachable!(),
        }
        match read_message(&[0]) {
            Err(MessageError::InvalidHeaderLength(length)) => assert_eq!(length, 1),
            _ => unreachable!(),
        }
        match read_message(&[0, 0]) {
            Err(MessageError::InvalidHeaderLength(length)) => assert_eq!(length, 2),
            _ => unreachable!(),
        }
    }

    #[test]
    fn read_message_invalid_advertised_message_length_test() {
        match read_message(&[0x04, 0, 7, 0, 0, 0, 0, 0]) {
            Err(MessageError::InvalidAdvertisedMessageLength(advertised_length, real_length)) => {
                assert_eq!(advertised_length, 7);
                assert_eq!(real_length, 5);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn read_message_unknown_message_type_test() {
        match read_message(&[0xff, 0, 0]) {
            Err(MessageError::UnknownMessageType(message_type)) => assert_eq!(message_type, 0xff),
            _ => unreachable!(),
        }
    }

    #[test]
    fn read_message_test() {
        let mut bytes = Vec::new();
        let message_length: u16 = 8;
        let milestone_index: u64 = 123456789;

        bytes.extend_from_slice(&[0x03]);
        bytes.extend_from_slice(&message_length.to_be_bytes());
        bytes.extend_from_slice(&milestone_index.to_be_bytes());

        let message = match read_message(&bytes) {
            Ok(MessageType::MilestoneRequest(message)) => message,
            _ => unreachable!(),
        };

        assert_eq!(message.index(), milestone_index);
    }
}
