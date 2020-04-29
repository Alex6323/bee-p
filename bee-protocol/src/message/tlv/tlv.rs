use crate::message::{
    Header,
    Message,
    MessageError,
    HEADER_SIZE,
    HEADER_TYPE_SIZE,
};

pub(crate) struct Tlv {}

impl Tlv {
    pub(crate) fn from_bytes<M: Message>(header: &Header, payload: &[u8]) -> Result<M, MessageError> {
        if header.message_type != M::ID {
            return Err(MessageError::InvalidAdvertisedType(header.message_type, M::ID));
        }

        if header.message_length as usize != payload.len() {
            return Err(MessageError::InvalidAdvertisedLength(
                header.message_length as usize,
                payload.len(),
            ));
        }

        if !M::size_range().contains(&payload.len()) {
            return Err(MessageError::InvalidPayloadLength(payload.len()));
        }

        Ok(M::from_bytes(payload))
    }

    pub(crate) fn into_bytes<M: Message>(message: M) -> Vec<u8> {
        let size = message.size();
        let mut bytes = vec![0u8; HEADER_SIZE + size];

        bytes[0] = M::ID;
        bytes[HEADER_TYPE_SIZE..HEADER_SIZE].copy_from_slice(&(size as u16).to_be_bytes());
        message.to_bytes(&mut bytes[HEADER_SIZE..]);

        bytes
    }
}
