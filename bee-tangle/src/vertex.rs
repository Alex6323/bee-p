use crate::TransactionRef;

use bee_transaction::{BundledTransaction as Transaction, Hash as TransactionHash, TransactionVertex};

use async_std::sync::Arc;

pub struct Vertex<Meta> {
    trunk: TransactionHash,
    branch: TransactionHash,
    transaction: TransactionRef,
    meta: Meta,
}

impl<Meta> Vertex<Meta> {
    pub fn new(transaction: Transaction, meta: Meta) -> Self {
        Self {
            trunk: transaction.trunk().clone(),
            branch: transaction.branch().clone(),
            transaction: TransactionRef(Arc::new(transaction)),
            meta,
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

    pub fn get_meta(&self) -> &Meta {
        &self.meta
    }

    pub fn get_meta_mut(&mut self) -> &mut Meta {
        &mut self.meta
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bee_test::transaction::create_random_tx;

    #[test]
    fn create_new_vertex() {
        let (hash, tx) = create_random_tx();

        let vtx = Vertex::new(tx.clone(), 0b0000_0001u8);

        assert_eq!(tx.trunk(), vtx.get_trunk());
        assert_eq!(tx.branch(), vtx.get_branch());
        assert_eq!(&tx, vtx.get_transaction());
        assert_eq!(meta, vtx.get_meta());
    }

    #[test]
    fn update_vertex_meta() {
        let (hash, tx) = create_random_tx();

        let mut vtx = Vertex::new(tx.clone(), 0b0000_0001u8);
        *vtx.get_meta_mut() = 0b1111_1110u8;

        assert_eq!(0b1111_1110u8, vtx.get_meta());
    }
}
