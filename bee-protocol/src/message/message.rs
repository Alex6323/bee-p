use std::ops::Range;

pub(crate) trait Message {
    type Error;

    fn id() -> u8
    where
        Self: std::marker::Sized;

    fn size_range() -> Range<usize>
    where
        Self: std::marker::Sized;

    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Error>
    where
        Self: std::marker::Sized;

    fn into_bytes(self) -> Vec<u8>;
}
