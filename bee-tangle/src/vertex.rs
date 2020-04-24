use crate::tangle::Tangle;

use bitflags::bitflags;

use std::ops::Deref;

use bee_bundle::{
    Hash,
    Transaction,
};

use async_std::sync::Arc;

/// A wrapper around `bee_bundle::Transaction` that allows sharing it safely across threads.
#[derive(Clone)]
pub struct TransactionRef(Arc<Transaction>);

impl Deref for TransactionRef {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

bitflags! {
    pub(crate) struct Flags: u8 {
        const IS_SOLID = 0x01;
        const IS_TAIL = 0x02;
    }
}

impl Flags {
    pub fn clear(&mut self) {
        self.bits = 0;
    }
}

pub(crate) struct Vertex {
    id: Hash,
    inner: TransactionRef,
    flags: Flags,
}

impl Vertex {
    pub fn from(transaction: Transaction, hash: Hash) -> Self {
        let flags = if transaction.is_tail() {
            Flags::IS_TAIL
        } else {
            Flags::empty()
        };

        Self {
            id: hash,
            inner: TransactionRef(Arc::new(transaction)),
            flags,
        }
    }

    pub fn get_id(&self) -> Hash {
        self.id
    }

    pub fn get_ref_to_inner(&self) -> TransactionRef {
        self.inner.clone()
    }

    pub fn set_solid(&mut self) {
        self.flags = Flags::IS_SOLID;
    }

    pub fn is_solid(&self) -> bool {
        self.flags == Flags::IS_SOLID
    }

    pub fn set_tail(&mut self) {
        self.flags = Flags::IS_TAIL;
    }

    pub fn is_tail(&self) -> bool {
        self.flags == Flags::IS_TAIL
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bee_test::transaction::create_random_tx;

    #[test]
    fn set_and_is_solid() {
        let (hash, tx) = create_random_tx();

        let mut vtx = Vertex::from(tx, hash);
        assert!(!vtx.is_solid());

        vtx.set_solid();
        assert!(vtx.is_solid())
    }
}
