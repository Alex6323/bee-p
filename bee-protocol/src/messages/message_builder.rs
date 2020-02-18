use crate::messages::errors::MessageError;
use crate::messages::handshake::Handshake;
use crate::messages::heartbeat::Heartbeat;
use crate::messages::legacy_gossip::LegacyGossip;
use crate::messages::milestone_request::MilestoneRequest;
use crate::messages::transaction_broadcast::TransactionBroadcast;
use crate::messages::transaction_request::TransactionRequest;
use crate::messages::MessageType;

use bee_network::Message;

use std::convert::TryInto;
use std::io::Read;

pub fn read_message<R: Read>(mut reader: R) -> Result<MessageType, MessageError> {
    let mut header_buffer = [0u8; 3];

    reader
        .read_exact(&mut header_buffer)
        .map_err(|_| MessageError::InvalidHeader)?;

    let message_type = header_buffer[0];
    let message_length = u16::from_be_bytes(
        header_buffer[1..3]
            .try_into()
            .map_err(|_| MessageError::InvalidHeader)?,
    );
    let mut message = vec![0u8; message_length as usize];

    reader
        .read_exact(&mut message)
        .map_err(|_| MessageError::InvalidMessage)?;

    match message_type {
        0x01 => Ok(MessageType::Handshake(Handshake::from_bytes(&message)?)),
        0x02 => Ok(MessageType::LegacyGossip(LegacyGossip::from_bytes(
            &message,
        )?)),
        0x03 => Ok(MessageType::MilestoneRequest(MilestoneRequest::from_bytes(
            &message,
        )?)),
        0x04 => Ok(MessageType::TransactionBroadcast(
            TransactionBroadcast::from_bytes(&message)?,
        )),
        0x05 => Ok(MessageType::TransactionRequest(
            TransactionRequest::from_bytes(&message)?,
        )),
        0x06 => Ok(MessageType::Heartbeat(Heartbeat::from_bytes(&message)?)),
        _ => Err(MessageError::InvalidMessageType(message_type)),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn read_message_invalid_header_length_test() {
        match read_message(&[][..]) {
            Err(MessageError::InvalidHeader) => (),
            _ => unreachable!(),
        }
        match read_message(&[0][..]) {
            Err(MessageError::InvalidHeader) => (),
            _ => unreachable!(),
        }
        match read_message(&[0, 0][..]) {
            Err(MessageError::InvalidHeader) => (),
            _ => unreachable!(),
        }
    }

    #[test]
    fn read_message_invalid_advertised_message_length_test() {
        match read_message(&[0x04, 0, 7, 0, 0, 0, 0, 0][..]) {
            Err(MessageError::InvalidMessage) => (),
            _ => unreachable!(),
        }
    }

    #[test]
    fn read_message_invalid_message_type_test() {
        match read_message(&[0xff, 0, 0][..]) {
            Err(MessageError::InvalidMessageType(message_type)) => assert_eq!(message_type, 0xff),
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

        let message = match read_message(&bytes[..]) {
            Ok(MessageType::MilestoneRequest(message)) => message,
            _ => unreachable!(),
        };

        assert_eq!(message.index(), milestone_index);
    }
}
