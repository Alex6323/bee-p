use crate::tangle::Tangle;

use std::sync::Arc;

use bee_bundle::{
    Hash,
    Transaction,
};

pub struct Vertex {
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

    pub fn get_transaction_ref(&self) -> Arc<Transaction> {
        Arc::clone(&self.transaction)
    }
}

#[derive(Copy, Clone)]
pub struct VertexMeta {
    id: Hash,
    trunk: Hash,
    branch: Hash,
}

#[derive(Copy, Clone)]
pub struct VertexRef {
    pub(crate) meta: VertexMeta,
    pub(crate) tangle: &'static Tangle,
}

impl VertexRef {
    pub fn get_transaction(&self) -> Option<Arc<Transaction>> {
        self.tangle.get_transaction(&self.meta.id)
    }

    pub fn get_trunk(&self) -> Option<Self> {
        self.tangle.get(&self.meta.trunk)
    }

    pub fn get_branch(&self) -> Option<Self> {
        self.tangle.get(&self.meta.branch)
    }
}

/*

/// A vertex within the Tangle. Each vertex represents a transaction and its associated metadata.
pub struct Vertex {
    hash: TxHash,
    tx: Tx,
    solid: bool,
}

impl Vertex {
    pub fn new(hash: TxHash, tx: Tx) -> Self {
        Self {
            hash,
            tx,
            solid: false,
        }
    }

    /// This method is private because all solidification should occur via the solidification
    /// algorithm automatically.
    fn set_solid(&mut self) {
        self.solid = true;
    }

    pub fn is_solid(&self) -> bool {
        self.solid
    }

    pub fn tx(&self) -> &Tx {
        &self.tx
    }
}

impl Deref for Vertex {
    type Target = Tx;

    fn deref(&self) -> &Tx {
        &self.tx
    }
}

pub struct VertexRef<'a> {
    tangle: &'a Tangle,
    hash: TxHash,
}

impl<'a> VertexRef<'a> {
    pub fn exists(&self) -> bool {
        self.tangle.contains(self.hash)
    }

    pub fn trunk(&'a self) -> Option<Self> {
        let trunk_hash = self.tx.trunk;
        self.tangle.get(trunk_hash)
    }

    pub fn branch(&'a self) -> Option<Self> {
        let branch_hash = self.tx.branch;
        self.tangle.get(branch_hash)
    }
}

impl<'a> Deref for VertexRef<'a> {
    type Target = Vertex;

    fn deref(&self) -> &Self::Target {
        self.tangle.vertices.get(&self.hash).unwrap()
    }
}

pub struct VertexRefMut<'a> {
    tangle: &'a mut Tangle,
    hash: TxHash,
}

impl<'a> VertexRefMut<'a> {
    pub fn trunk(&'a mut self) -> Option<Self> {
        let trunk_hash = self.tx.trunk;
        self.tangle.get_mut(trunk_hash)
    }

    pub fn branch(&'a mut self) -> Option<Self> {
        let branch_hash = self.tx.branch;
        self.tangle.get_mut(branch_hash)
    }
}

impl<'a> Deref for VertexRefMut<'a> {
    type Target = Vertex;

    fn deref(&self) -> &Self::Target {
        self.tangle.vertices.get(&self.hash).unwrap()
    }
}

impl<'a> DerefMut for VertexRefMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.tangle.vertices.get_mut(&self.hash).unwrap()
    }
}

impl<'a> VertexRefMut<'a> {
    pub fn do_for(&self, f: impl FnOnce(&Vertex)) {
        f(self.tangle.vertices.get(&self.hash).unwrap());
    }

    pub fn do_for_mut(&mut self, f: impl FnOnce(&mut Vertex)) {
        f(self.tangle.vertices.get_mut(&self.hash).unwrap());
    }
}
*/
