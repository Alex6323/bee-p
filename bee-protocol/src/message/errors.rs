#[derive(Debug)]
pub(crate) enum MessageError {
    InvalidAdvertisedLengthBytes([u8; 2]),
    InvalidAdvertisedLength(usize, usize),
    InvalidPayloadLength(usize),
    InvalidPayloadField,
}
