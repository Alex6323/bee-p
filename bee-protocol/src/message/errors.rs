#[derive(Debug)]
pub enum ProtocolMessageError {
    InvalidHeaderLength(usize),
    InvalidAdvertisedLengthBytes([u8; 2]),
    InvalidAdvertisedLength(usize, usize),
    InvalidMessageLength(usize),
    InvalidMessageField,
}
