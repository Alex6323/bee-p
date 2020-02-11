use crate::messages::handshake::Handshake;
use crate::messages::header::Header;
use crate::messages::Message;
use crate::messages::MessageType;

pub fn create_message(bytes: &[u8]) -> impl Message {
    let header = Header::from_bytes(&bytes[Header::size_range()]);

    match bytes[0] {
        0 => Handshake::new(),
        _ => unreachable!(),
    }
}
