pub enum MessageError {
    UnknownMessageType(u8),
    InvalidMessageLength(usize),
}
