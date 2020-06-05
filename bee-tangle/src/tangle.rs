use crate::{vertex::Vertex, TransactionRef as TxRef};

use bee_transaction::{BundledTransaction as Tx, Hash as TxHash, TransactionVertex};

use dashmap::{mapref::entry::Entry, DashMap};

pub struct Tangle<T> {
    pub vertices: DashMap<TxHash, Vertex<T>>,
    pub approvers: DashMap<TxHash, Vec<TxHash>>,
}

impl<T> Default for Tangle<T> {
    fn default() -> Self {
        Self {
            vertices: DashMap::new(),
            approvers: DashMap::new(),
        }
    }
}

impl<T> Tangle<T> {
    /// Creates a new Tangle.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a transaction, and returns a thread-safe reference to it. If the transaction was new `true` is returned,
    /// otherwise `false`.
    pub fn insert_transaction(&self, transaction: Tx, hash: TxHash, metadata: T) -> (TxRef, bool) {
        self.add_approver(*transaction.trunk(), hash);

        if transaction.trunk() != transaction.branch() {
            self.add_approver(*transaction.branch(), hash);
        }

        match self.vertices.entry(hash) {
            Entry::Occupied(entry) => (entry.get().get_transaction().clone(), false),
            Entry::Vacant(entry) => {
                let vtx = Vertex::new(transaction, metadata);
                let tx_ref = vtx.get_transaction().clone();
                entry.insert(vtx);
                (tx_ref, true)
            }
        }
    }

    #[inline]
    fn add_approver(&self, approvee: TxHash, approver: TxHash) {
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

    pub fn get_transaction(&self, hash: &TxHash) -> Option<TxRef> {
        self.vertices.get(hash).map(|vtx| vtx.value().get_transaction().clone())
    }

    /// Returns whether the transaction is stored in the Tangle.
    pub fn contains_transaction(&self, hash: &TxHash) -> bool {
        self.vertices.contains_key(hash)
    }

    pub fn update_metadata(&self, hash: &TxHash, metadata: T) {
        self.vertices.get_mut(hash).map(|mut vtx| {
            let vtx = vtx.value_mut();
            *vtx.get_metadata_mut() = metadata;
        });
    }

    /// Returns the current size of the Tangle.
    pub fn size(&self) -> usize {
        self.vertices.len()
    }

    #[cfg(test)]
    pub(crate) fn num_approvers(&self, hash: &TxHash) -> usize {
        self.approvers.get(hash).map_or(0, |r| r.value().len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bee_test::transaction::create_random_tx;

    #[test]
    fn new_tangle() {
        let _: Tangle<u8> = Tangle::new();
    }

    #[test]
    fn insert_and_contains() {
        let tangle = Tangle::new();

        let (hash, tx) = create_random_tx();

        let (_, is_new) = tangle.insert_transaction(tx.clone(), hash.clone(), ());

        assert!(is_new);
        assert_eq!(1, tangle.size());
        assert!(tangle.contains_transaction(&hash));

        let (_, is_new) = tangle.insert_transaction(tx, hash, ());

        assert!(!is_new);
        assert_eq!(1, tangle.size());
        assert!(tangle.contains_transaction(&hash));
    }
}
