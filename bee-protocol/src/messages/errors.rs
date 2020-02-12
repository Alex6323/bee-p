#[derive(Debug)]
pub enum MessageError {
    UnknownMessageType(u8),
    InvalidHeaderLength(usize),
    InvalidAdvertisedMessageLength(usize, usize),
    InvalidMessageLength(usize),
}
