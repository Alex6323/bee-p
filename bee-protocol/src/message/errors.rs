#[derive(Debug)]
pub enum ProtocolMessageError {
    InvalidHeader,
    InvalidMessage,
    InvalidMessageType(u8),
    InvalidMessageLength(usize),
}
