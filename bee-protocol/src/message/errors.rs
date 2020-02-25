#[derive(Debug)]
pub(crate) enum MessageError {
    InvalidHeaderLength(usize),
    InvalidAdvertisedLengthBytes([u8; 2]),
    InvalidAdvertisedLength(usize, usize),
    InvalidMessageLength(usize),
    InvalidMessageField,
}
