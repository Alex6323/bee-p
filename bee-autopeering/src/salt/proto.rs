use super::Salt;

pub struct ProtoSalt {
    bytes: Vec<u8>,
    expiration_time: u64,
}

impl From<Salt> for ProtoSalt {
    fn from(salt: Salt) -> Self {
        Self {
            // TODO
            bytes: Vec::new(),
            expiration_time: 0,
        }
    }
}
