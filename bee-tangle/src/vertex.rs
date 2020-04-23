use crate::tangle::Tangle;

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

pub(crate) struct Vertex {
    pub(crate) id: Hash,
    transaction: Arc<Transaction>,
}

impl Vertex {
    pub fn from(transaction: Transaction, hash: Hash) -> Self {
        Self {
            id: hash,
            transaction: Arc::new(transaction),
        }
    }

    pub fn get_id(&self) -> Hash {
        self.id
    }

    pub fn get_transaction(&self) -> TransactionRef {
        TransactionRef(Arc::clone(&self.transaction))
    }
}
