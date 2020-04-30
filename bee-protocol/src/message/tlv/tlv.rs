//! Type-length-value encoding/decoding.

use crate::message::{
    Header,
    Message,
    MessageError,
    HEADER_SIZE,
};

/// Since the following methods have very common names, `from_bytes` and `into_bytes`, the sole purpose of this struct
/// is to give them a proper namespace to avoid confusion.
pub(crate) struct Tlv {}

impl Tlv {
    /// Deserializes a TLV header and a bytes buffer into a message.
    ///
    /// # Arguments
    ///
    /// * `header`  -   The TLV header to deserialize from.
    /// * `bytes`   -   The bytes buffer to deserialize from.
    ///
    /// # Errors
    ///
    /// * The advertised message type doesn't match the required message type.
    /// * The advertised message length doesn't match the buffer length.
    /// * The buffer length is not within the allowed size range of the required message type.
    pub(crate) fn from_bytes<M: Message>(header: &Header, bytes: &[u8]) -> Result<M, MessageError> {
        if header.message_type != M::ID {
            return Err(MessageError::InvalidAdvertisedType(header.message_type, M::ID));
        }

        if header.message_length as usize != bytes.len() {
            return Err(MessageError::InvalidAdvertisedLength(
                header.message_length as usize,
                bytes.len(),
            ));
        }

        if !M::size_range().contains(&bytes.len()) {
            return Err(MessageError::InvalidLength(bytes.len()));
        }

        Ok(M::from_bytes(bytes))
    }

    /// Serializes a TLV header and a message into a bytes buffer.
    ///
    /// # Arguments
    ///
    /// * `message` -   The message to serialize.
    pub(crate) fn into_bytes<M: Message>(message: M) -> Vec<u8> {
        let size = message.size();
        let mut bytes = vec![0u8; HEADER_SIZE + size];
        let (header, payload) = bytes.split_at_mut(HEADER_SIZE);

        header[0] = M::ID;
        header[1..].copy_from_slice(&(size as u16).to_be_bytes());
        message.into_bytes(payload);

        bytes
    }
}
