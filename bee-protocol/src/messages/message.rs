use std::ops::Range;

pub trait Message {
    fn size_range() -> Range<usize>;

    fn from_bytes(bytes: &[u8]) -> Self;

    fn to_bytes(self) -> Vec<u8>;
}
