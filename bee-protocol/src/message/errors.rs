#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum MessageError {
    InvalidAdvertisedType(u8, u8),
    InvalidAdvertisedLength(usize, usize),
    InvalidPayloadLength(usize),
}
