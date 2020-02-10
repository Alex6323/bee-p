use crate::messages::message::Message;

pub struct HandShake {}

impl Message for HandShake {
    fn from_bytes(_bytes: &[u8]) -> Self {
        Self {}
    }

    fn to_bytes() -> Vec<u8> {
        [].to_vec()
    }
}
