use crate::tangle::Tangle;

use bee_bundle::{
    Hash,
    Transaction,
};

use async_std::sync::Arc;

/// A wrapper around `bee_bundle::Transaction` that allows sharing it across threads.
#[derive(Clone)]
pub struct TransactionRef(Arc<Transaction>);

impl std::ops::Deref for TransactionRef {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

pub(crate) struct Vertex {
    pub(crate) meta: VertexMeta,
    transaction: Arc<Transaction>,
}

impl Vertex {
    pub fn from(transaction: Transaction, hash: Hash) -> Self {
        Self {
            meta: VertexMeta {
                id: hash,
                trunk: *transaction.trunk(),
                branch: *transaction.branch(),
            },
            transaction: Arc::new(transaction),
        }
    }

    pub fn get_transaction(&self) -> TransactionRef {
        TransactionRef(Arc::clone(&self.transaction))
    }

    pub fn get_id(&self) -> Hash {
        self.meta.id
    }
}

#[derive(Copy, Clone)]
pub(crate) struct VertexMeta {
    pub(crate) id: Hash,
    pub(crate) trunk: Hash,
    pub(crate) branch: Hash,
}