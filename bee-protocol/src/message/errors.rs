#[derive(Debug)]
pub enum ProtocolMessageError {
    InvalidHeader,
    InvalidMessage,
    InvalidMessageField,
    InvalidMessageType(u8),
    InvalidMessageLength(usize),
}
