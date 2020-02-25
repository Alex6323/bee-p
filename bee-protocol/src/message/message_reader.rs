use crate::message::{
    Handshake, Heartbeat, LegacyGossip, Message, MessageError, MilestoneRequest,
    ProtocolMessageType, TransactionBroadcast, TransactionRequest,
};

use async_std::io::Read;
use async_std::prelude::*;
use async_trait::async_trait;

use std::convert::TryInto;

#[async_trait]
pub(crate) trait MessageReader {
    type MessageType;
    type Error;

    async fn read<R>(reader: R) -> Result<Self::MessageType, Self::Error>
    where
        R: Read + std::marker::Unpin + std::marker::Send;
}

pub(crate) struct ProtocolMessageReader {}

#[async_trait]
impl MessageReader for ProtocolMessageReader {
    type MessageType = ProtocolMessageType;
    type Error = MessageError;

    async fn read<R>(mut reader: R) -> Result<Self::MessageType, Self::Error>
    where
        R: Read + std::marker::Unpin + std::marker::Send,
    {
        let mut header_buffer = [0u8; 3];

        reader
            .read_exact(&mut header_buffer)
            .await
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
            .await
            .map_err(|_| MessageError::InvalidMessage)?;

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
            _ => Err(MessageError::InvalidMessageType(message_type)),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use futures::executor::block_on;

    #[test]
    fn read_message_invalid_header_length_test() {
        match block_on(ProtocolMessageReader::read(&[][..])) {
            Err(MessageError::InvalidHeader) => (),
            _ => unreachable!(),
        }
        match block_on(ProtocolMessageReader::read(&[0][..])) {
            Err(MessageError::InvalidHeader) => (),
            _ => unreachable!(),
        }
        match block_on(ProtocolMessageReader::read(&[0, 0][..])) {
            Err(MessageError::InvalidHeader) => (),
            _ => unreachable!(),
        }
    }

    #[test]
    fn read_message_invalid_advertised_message_length_test() {
        match block_on(ProtocolMessageReader::read(
            &[0x04, 0, 7, 0, 0, 0, 0, 0][..],
        )) {
            Err(MessageError::InvalidMessage) => (),
            _ => unreachable!(),
        }
    }

    #[test]
    fn read_message_invalid_message_type_test() {
        match block_on(ProtocolMessageReader::read(&[0xff, 0, 0][..])) {
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

        let message = match block_on(ProtocolMessageReader::read(&bytes[..])) {
            Ok(ProtocolMessageType::MilestoneRequest(message)) => message,
            _ => unreachable!(),
        };

        assert_eq!(message.index(), milestone_index);
    }
}
