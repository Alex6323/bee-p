#[derive(Debug)]
pub enum MessageError {
    InvalidAdvertisedLengthBytes([u8; 2]),
    InvalidAdvertisedType(u8, u8),
    InvalidAdvertisedLength(usize, usize),
    InvalidPayloadLength(usize),
    InvalidPayloadField,
}
