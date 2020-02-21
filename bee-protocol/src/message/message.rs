use crate::message::ProtocolMessageError;

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

    fn into_bytes(self) -> Vec<u8>;
}
