use crate::tangle::Tangle;

use bitflags::bitflags;

use std::ops::Deref;

use bee_bundle::{Hash, Transaction};

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
        const SOLID = 0b0000_0001;
        const TAIL = 0b0000_0010;
        const REQUESTED = 0b0000_0100;
        const MILESTONE = 0b0000_1000;
    }
}

pub struct Vertex {
    id: Hash,
    inner: TransactionRef,
    flags: Flags,
}

impl Vertex {
    pub fn from(transaction: Transaction, hash: Hash) -> Self {
        let flags = if transaction.is_tail() {
            Flags::TAIL
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

    pub fn is_solid(&self) -> bool {
        self.flags.contains(Flags::SOLID)
    }

    pub fn set_solid(&mut self) {
        self.flags.insert(Flags::SOLID);
    }

    pub fn is_tail(&self) -> bool {
        self.flags.contains(Flags::TAIL)
    }

    pub fn is_requested(&self) -> bool {
        self.flags.contains(Flags::REQUESTED)
    }

    pub fn set_requested(&mut self) {
        self.flags.insert(Flags::REQUESTED);
    }

    pub fn is_milestone(&self) -> bool {
        self.flags.contains(Flags::MILESTONE)
    }

    pub fn set_milestone(&mut self) {
        self.flags.insert(Flags::MILESTONE);
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
