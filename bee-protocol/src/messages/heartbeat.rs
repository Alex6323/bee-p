use crate::messages::message::Message;

pub struct Heartbeat {}

impl Message for Heartbeat {
    fn from_bytes(_bytes: &[u8]) -> Self {
        Self {}
    }

    fn to_bytes() -> Vec<u8> {
        [].to_vec()
    }
}
