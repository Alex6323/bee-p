use crate::messages::errors::MessageError;

use std::ops::Range;

pub trait Message {
    fn size_range() -> Range<usize>;

    fn from_bytes(bytes: &[u8]) -> Result<Self, MessageError>
    where
        Self: std::marker::Sized;

    fn to_bytes(self) -> Vec<u8>;
}
