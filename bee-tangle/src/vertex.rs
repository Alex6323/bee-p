use crate::TransactionRef;

use bee_transaction::{BundledTransaction as Transaction, Hash as TransactionHash, TransactionVertex};

use async_std::sync::Arc;

pub struct Vertex<T> {
    trunk: TransactionHash,
    branch: TransactionHash,
    transaction: TransactionRef,
    metadata: T,
}

impl<T> Vertex<T> {
    pub fn new(transaction: Transaction, metadata: T) -> Self {
        Self {
            trunk: transaction.trunk().clone(),
            branch: transaction.branch().clone(),
            transaction: TransactionRef(Arc::new(transaction)),
            metadata,
        }
    }

    pub fn get_trunk(&self) -> &TransactionHash {
        &self.trunk
    }

    pub fn get_branch(&self) -> &TransactionHash {
        &self.branch
    }

    pub fn get_transaction(&self) -> &TransactionRef {
        &self.transaction
    }

    pub fn get_metadata(&self) -> &T {
        &self.metadata
    }

    pub fn get_metadata_mut(&mut self) -> &mut T {
        &mut self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bee_test::transaction::create_random_tx;

    #[test]
    fn create_new_vertex() {
        let (_, tx) = create_random_tx();
        let metadata = 0b0000_0001u8;

        let vtx = Vertex::new(tx.clone(), metadata);

        assert_eq!(tx.trunk(), vtx.get_trunk());
        assert_eq!(tx.branch(), vtx.get_branch());
        assert_eq!(tx, **vtx.get_transaction());
        assert_eq!(metadata, *vtx.get_metadata());
    }

    #[test]
    fn update_vertex_meta() {
        let (_, tx) = create_random_tx();

        let mut vtx = Vertex::new(tx, 0b0000_0001u8);
        *vtx.get_metadata_mut() = 0b1111_1110u8;

        assert_eq!(0b1111_1110u8, *vtx.get_metadata());
    }
}
