use crate::{vertex::Vertex, TransactionRef as TxRef};

use bee_transaction::{BundledTransaction as Tx, Hash as TxHash, TransactionVertex};

use dashmap::{mapref::entry::Entry, DashMap};

pub struct Tangle<T>
where
    T: Clone + Copy,
{
    // 'map_hash_vertex'
    pub vertices: DashMap<TxHash, Vertex<T>>,
    // TODO: rename this to 'map_parent_children'
    pub approvers: DashMap<TxHash, Vec<TxHash>>,
    // TODO: add 'tips' DashSet for fast tip selection
}

impl<T> Default for Tangle<T>
where
    T: Clone + Copy,
{
    fn default() -> Self {
        Self {
            vertices: DashMap::new(),
            approvers: DashMap::new(),
        }
    }
}

impl<T> Tangle<T>
where
    T: Clone + Copy,
{
    /// Creates a new Tangle.
    pub fn new() -> Self {
        Self::default()
    }

    // TODO: maybe swap 'hash' and 'metadata'
    /// Inserts a transaction, and returns a thread-safe reference to it. If the transaction was new `true` is returned,
    /// otherwise `false`.
    pub fn insert(&self, data: Tx, hash: TxHash, metadata: T) -> Option<TxRef> {
        self.add_approver(*data.trunk(), hash);

        if data.trunk() != data.branch() {
            self.add_approver(*data.branch(), hash);
        }

        match self.vertices.entry(hash) {
            Entry::Occupied(_) => None,
            Entry::Vacant(entry) => {
                let vtx = Vertex::new(data, metadata);
                let txref = vtx.get_transaction().clone();
                entry.insert(vtx);
                Some(txref)
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

    // TODO: docs
    // TODO: closure?
    pub fn get_data(&self, hash: &TxHash) -> Option<TxRef> {
        self.vertices.get(hash).map(|vtx| vtx.value().get_transaction().clone())
    }

    pub fn get_metadata(&self, hash: &TxHash) -> Option<T> {
        self.vertices.get(hash).map(|vtx| *vtx.value().get_metadata())
    }

    /// Returns whether the transaction is stored in the Tangle.
    pub fn contains(&self, hash: &TxHash) -> bool {
        self.vertices.contains_key(hash)
    }

    /// Updates the metadata of a particular vertex.
    pub fn update(&self, hash: &TxHash, metadata: T) {
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

        let (_, is_new) = tangle.insert(tx.clone(), hash.clone(), ());

        assert!(is_new);
        assert_eq!(1, tangle.size());
        assert!(tangle.contains(&hash));

        let (_, is_new) = tangle.insert_transaction(tx, hash, ());

        assert!(!is_new);
        assert_eq!(1, tangle.size());
        assert!(tangle.contains_transaction(&hash));
    }
}
