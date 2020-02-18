use crate::messages::errors::MessageError;
use crate::messages::message::Message;

use std::ops::Range;

const TRANSACTION_BROADCAST_VARIABLE_MIN_SIZE: usize = 292;
const TRANSACTION_BROADCAST_VARIABLE_MAX_SIZE: usize = 1604;

#[derive(Clone)]
pub struct TransactionBroadcast {
    transaction: Vec<u8>,
}

impl TransactionBroadcast {
    pub fn new(transaction: &[u8]) -> Self {
        Self {
            transaction: transaction.to_vec(),
        }
    }

    pub fn transaction(&self) -> &Vec<u8> {
        &self.transaction
    }
}

impl Message for TransactionBroadcast {
    type Error = MessageError;

    fn size_range() -> Range<usize> {
        (TRANSACTION_BROADCAST_VARIABLE_MIN_SIZE)..(TRANSACTION_BROADCAST_VARIABLE_MAX_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, MessageError> {
        if !Self::size_range().contains(&bytes.len()) {
            Err(MessageError::InvalidMessageLength(bytes.len()))?;
        }

        Ok(Self {
            transaction: bytes.to_vec(),
        })
    }

    fn into_bytes(self) -> Vec<u8> {
        self.transaction
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn size_range_test() {
        assert_eq!(TransactionBroadcast::size_range().contains(&291), false);
        assert_eq!(TransactionBroadcast::size_range().contains(&292), true);
        assert_eq!(TransactionBroadcast::size_range().contains(&293), true);

        assert_eq!(TransactionBroadcast::size_range().contains(&1603), true);
        assert_eq!(TransactionBroadcast::size_range().contains(&1604), true);
        assert_eq!(TransactionBroadcast::size_range().contains(&1605), false);
    }

    #[test]
    fn from_bytes_invalid_length_test() {
        match TransactionBroadcast::from_bytes(&[0; 291]) {
            Err(MessageError::InvalidMessageLength(length)) => assert_eq!(length, 291),
            _ => unreachable!(),
        }
        match TransactionBroadcast::from_bytes(&[0; 1605]) {
            Err(MessageError::InvalidMessageLength(length)) => assert_eq!(length, 1605),
            _ => unreachable!(),
        }
    }

    #[test]
    fn new_into_from_test() {
        let transaction: Vec<u8> = (500..1000).map(|i| i as u8).collect();
        let message_from = TransactionBroadcast::new(&transaction);
        let message_to = TransactionBroadcast::from_bytes(&message_from.into_bytes()).unwrap();

        assert_eq!(message_to.transaction(), &transaction);
    }
}
