
use serde_json::Value;
use std::fmt;
use bee_transaction::{Hash, BundledTransactionField};
use bee_ternary::{TritBuf, T1B1Buf, TryteBuf};

fn deserialize_tryte_str(tryte_str: &str) -> Result<TritBuf, DeserializationError> {
    match TryteBuf::try_from_str(tryte_str) {
        Ok(buf) => Ok(buf.as_trits().encode::<T1B1Buf>()),
        Err(_err) => Err(DeserializationError::new(String::from("string contains invalid characters")))
    }
}

pub fn deserialize_hash(hash: &Value) -> Result<Hash, DeserializationError> {
    match hash.as_str() {
        Some(hash) => {
            match Hash::try_from_inner(deserialize_tryte_str(hash)?) {
                Ok(hash) => Ok(hash),
                Err(_err) => Err(DeserializationError::new(String::from("hash has invalid size")))
            }
        }
        None => {
            Err(DeserializationError::new(String::from("hash needs to be string")))
        }
    }
}

pub fn deserialize_hash_array(hash_array: Option<&Vec<Value>>) -> Result<Vec<Hash>, DeserializationError> {
    match hash_array {
        Some(hashes) => {
            let mut ret = Vec::new();
            for hash in hashes {
                ret.push(deserialize_hash(hash)?);
            }
            Ok(ret)
        }
        None => {
            Err(DeserializationError::new(String::from("no hashes provided")))
        }
    }
}

pub struct DeserializationError {
    pub msg: String,
}

impl DeserializationError {
    fn new(msg: String) -> Self {
        Self { msg }
    }
}

impl fmt::Display for DeserializationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}