use crate::message::ProtocolMessageError;

use std::convert::TryInto;
use std::ops::Range;

pub(crate) trait Message {
    fn id() -> u8
    where
        Self: std::marker::Sized;

    fn size_range() -> Range<usize>
    where
        Self: std::marker::Sized;

    fn from_bytes(bytes: &[u8]) -> Result<Self, ProtocolMessageError>
    where
        Self: std::marker::Sized;

    fn from_full_bytes(bytes: &[u8]) -> Result<Self, ProtocolMessageError>
    where
        Self: std::marker::Sized,
    {
        if bytes.len() < 3 {
            Err(ProtocolMessageError::InvalidHeaderLength(bytes.len()))?;
        }

        let message_length = u16::from_be_bytes(bytes[1..3].try_into().map_err(|_| {
            ProtocolMessageError::InvalidAdvertisedLengthBytes([bytes[1], bytes[2]])
        })?);

        if message_length as usize != bytes[3..].len() {
            Err(ProtocolMessageError::InvalidAdvertisedLength(
                message_length as usize,
                bytes[3..].len(),
            ))?;
        }

        Self::from_bytes(&bytes[3..])
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
