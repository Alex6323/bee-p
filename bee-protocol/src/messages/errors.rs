#[derive(Debug)]
pub enum MessageError {
    InvalidHeader,
    InvalidMessage,
    InvalidMessageType(u8),
    InvalidMessageLength(usize),
}
