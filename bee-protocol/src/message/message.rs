use crate::message::MessageError;

use std::convert::TryInto;
use std::ops::Range;

pub(crate) trait Message {
    fn id() -> u8
    where
        Self: std::marker::Sized;

    fn size_range() -> Range<usize>
    where
        Self: std::marker::Sized;

    fn from_bytes(bytes: &[u8]) -> Result<Self, MessageError>
    where
        Self: std::marker::Sized;

    fn from_full_bytes(header_bytes: &[u8], payload_bytes: &[u8]) -> Result<Self, MessageError>
    where
        Self: std::marker::Sized,
    {
        if header_bytes.len() < 3 {
            Err(MessageError::InvalidHeaderLength(header_bytes.len()))?;
        }

        let payload_length = u16::from_be_bytes(header_bytes[1..3].try_into().map_err(|_| {
            MessageError::InvalidAdvertisedLengthBytes([header_bytes[1], header_bytes[2]])
        })?);

        if payload_length as usize != payload_bytes.len() {
            Err(MessageError::InvalidAdvertisedLength(
                payload_length as usize,
                payload_bytes.len(),
            ))?;
        }

        Self::from_bytes(payload_bytes)
    }

    fn into_bytes(self) -> Vec<u8>;

    fn into_full_bytes(self) -> Vec<u8>
    where
        Self: std::marker::Sized,
    {
        let bytes = self.into_bytes();
        let mut full_bytes = Vec::new();

        full_bytes.push(Self::id());
        full_bytes.extend(&(bytes.len() as u16).to_be_bytes());
        full_bytes.extend(bytes);

        full_bytes
    }
}
