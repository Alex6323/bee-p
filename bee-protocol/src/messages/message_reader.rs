use crate::messages::errors::ProtocolMessageError;
use crate::messages::handshake::Handshake;
use crate::messages::heartbeat::Heartbeat;
use crate::messages::legacy_gossip::LegacyGossip;
use crate::messages::milestone_request::MilestoneRequest;
use crate::messages::transaction_broadcast::TransactionBroadcast;
use crate::messages::transaction_request::TransactionRequest;
use crate::messages::ProtocolMessageType;

use bee_network::{Message, MessageReader};

use std::convert::TryInto;
use std::io::Read;

pub struct ProtocolMessageReader {}

impl MessageReader for ProtocolMessageReader {
    type MessageType = ProtocolMessageType;
    type Error = ProtocolMessageError;

    fn read<R: Read>(mut reader: R) -> Result<Self::MessageType, Self::Error> {
        let mut header_buffer = [0u8; 3];

        reader
            .read_exact(&mut header_buffer)
            .map_err(|_| ProtocolMessageError::InvalidHeader)?;

        let message_type = header_buffer[0];
        let message_length = u16::from_be_bytes(
            header_buffer[1..3]
                .try_into()
                .map_err(|_| ProtocolMessageError::InvalidHeader)?,
        );
        let mut message = vec![0u8; message_length as usize];

        reader
            .read_exact(&mut message)
            .map_err(|_| ProtocolMessageError::InvalidMessage)?;

        match message_type {
            0x01 => Ok(ProtocolMessageType::Handshake(Handshake::from_bytes(
                &message,
            )?)),
            0x02 => Ok(ProtocolMessageType::LegacyGossip(LegacyGossip::from_bytes(
                &message,
            )?)),
            0x03 => Ok(ProtocolMessageType::MilestoneRequest(
                MilestoneRequest::from_bytes(&message)?,
            )),
            0x04 => Ok(ProtocolMessageType::TransactionBroadcast(
                TransactionBroadcast::from_bytes(&message)?,
            )),
            0x05 => Ok(ProtocolMessageType::TransactionRequest(
                TransactionRequest::from_bytes(&message)?,
            )),
            0x06 => Ok(ProtocolMessageType::Heartbeat(Heartbeat::from_bytes(
                &message,
            )?)),
            _ => Err(ProtocolMessageError::InvalidMessageType(message_type)),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn read_message_invalid_header_length_test() {
        match ProtocolMessageReader::read(&[][..]) {
            Err(ProtocolMessageError::InvalidHeader) => (),
            _ => unreachable!(),
        }
        match ProtocolMessageReader::read(&[0][..]) {
            Err(ProtocolMessageError::InvalidHeader) => (),
            _ => unreachable!(),
        }
        match ProtocolMessageReader::read(&[0, 0][..]) {
            Err(ProtocolMessageError::InvalidHeader) => (),
            _ => unreachable!(),
        }
    }
    #[test]
    fn read_message_invalid_advertised_message_length_test() {
        match ProtocolMessageReader::read(&[0x04, 0, 7, 0, 0, 0, 0, 0][..]) {
            Err(ProtocolMessageError::InvalidMessage) => (),
            _ => unreachable!(),
        }
    }
    #[test]
    fn read_message_invalid_message_type_test() {
        match ProtocolMessageReader::read(&[0xff, 0, 0][..]) {
            Err(ProtocolMessageError::InvalidMessageType(message_type)) => {
                assert_eq!(message_type, 0xff)
            }
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

        let message = match ProtocolMessageReader::read(&bytes[..]) {
            Ok(ProtocolMessageType::MilestoneRequest(message)) => message,
            _ => unreachable!(),
        };

        assert_eq!(message.index(), milestone_index);
    }
}
