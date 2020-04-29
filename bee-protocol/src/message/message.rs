use crate::message::MessageError;

use std::ops::Range;

pub(crate) trait Message {
    const ID: u8;

    fn size_range() -> Range<usize>;

    fn from_bytes(bytes: &[u8]) -> Self;

    fn size(&self) -> usize;

    fn to_bytes(self, bytes: &mut [u8]);
}
