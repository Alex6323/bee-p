use crate::{vertex::Vertex, TransactionRef};

use bee_transaction::{BundledTransaction as Transaction, Hash as TransactionHash, TransactionVertex};

use dashmap::{mapref::entry::Entry, DashMap};

// TODO: Should we even add Key and Value type parameters?
pub struct Tangle<Meta> {
    pub vertices: DashMap<TransactionHash, Vertex<Meta>>,
    pub approvers: DashMap<TransactionHash, Vec<TransactionHash>>,
}

impl<Meta> Default for Tangle<Meta> {
    fn default() -> Self {
        Self {
            vertices: DashMap::new(),
            approvers: DashMap::new(),
        }
    }
}

impl<Meta> Tangle<Meta> {
    pub fn new() -> Self {
        Self {
            vertices: DashMap::new(),
            approvers: DashMap::new(),
        }
    }

    pub fn insert_transaction(
        &self,
        transaction: Transaction,
        hash: TransactionHash,
        meta: Meta,
    ) -> (TransactionRef, bool) {
        self.add_approver(*transaction.trunk(), hash);

        if transaction.trunk() != transaction.branch() {
            self.add_approver(*transaction.branch(), hash);
        }

        match self.vertices.entry(hash) {
            Entry::Occupied(entry) => (entry.get().get_transaction().clone(), false),
            Entry::Vacant(entry) => {
                let vtx = Vertex::new(transaction, meta);
                let tx_ref = vtx.get_transaction().clone();
                entry.insert(vtx);
                (tx_ref, true)
            }
        }
    }

    #[inline]
    fn add_approver(&self, approvee: TransactionHash, approver: TransactionHash) {
        match self.approvers.entry(approvee) {
            Entry::Occupied(mut entry) => {
                let approvers = entry.get_mut();
                approvers.push(approver);
            }
            Entry::Vacant(entry) => {
                entry.insert(vec![approver]);
            }
        }
    }

    pub fn get_transaction(&self, hash: &TransactionHash) -> Option<TransactionRef> {
        self.vertices.get(hash).map(|vtx| vtx.value().get_transaction().clone())
    }

    pub fn update_meta(&self, hash: &TransactionHash, meta: Meta) {
        self.vertices.get_mut(hash).map(|mut vtx| {
            let vtx = vtx.value_mut();
            *vtx.get_meta_mut() = meta;
        });
    }
}
