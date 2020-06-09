use crate::{vertex::Vertex, TransactionRef as TxRef};

use bee_transaction::{BundledTransaction as Tx, Hash as TxHash, TransactionVertex};

use dashmap::{mapref::entry::Entry, DashMap, DashSet};

use std::collections::HashSet;

/// A foundational, thread-safe graph datastructure to represent the IOTA Tangle.
pub struct Tangle<T>
where
    T: Clone + Copy,
{
    pub(crate) vertices: DashMap<TxHash, Vertex<T>>,
    pub(crate) children: DashMap<TxHash, HashSet<TxHash>>,
    pub(crate) tips: DashSet<TxHash>,
}

impl<T> Default for Tangle<T>
where
    T: Clone + Copy,
{
    fn default() -> Self {
        Self {
            vertices: DashMap::new(),
            children: DashMap::new(),
            tips: DashSet::new(),
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

    /// Inserts a transaction, and returns a thread-safe reference to it in case it didn't already exist.
    pub fn insert(&self, transaction: Tx, hash: TxHash, metadata: T) -> Option<TxRef> {
        match self.vertices.entry(hash) {
            Entry::Occupied(_) => None,
            Entry::Vacant(entry) => {
                self.add_child(*transaction.trunk(), hash);
                self.add_child(*transaction.branch(), hash);

                self.tips.remove(transaction.trunk());
                self.tips.remove(transaction.branch());

                let has_children = |hash| self.children.contains_key(hash);

                if !has_children(&hash) {
                    self.tips.insert(hash);
                } else {
                    self.tips.remove(&hash);
                }

                let vtx = Vertex::new(transaction, metadata);
                let tx = vtx.transaction().clone();
                entry.insert(vtx);
                Some(tx)
            }
        }
    }

    #[inline]
    fn add_child(&self, parent: TxHash, child: TxHash) {
        match self.children.entry(parent) {
            Entry::Occupied(mut entry) => {
                let children = entry.get_mut();
                children.insert(child);
            }
            Entry::Vacant(entry) => {
                // TODO: find a good value for pre-allocation
                let mut children = HashSet::new();
                children.insert(parent);
                entry.insert(children);
            }
        }
    }

    /// Get the data of a vertex associated with the given `hash`.
    pub fn get(&self, hash: &TxHash) -> Option<TxRef> {
        self.vertices.get(hash).map(|vtx| vtx.value().transaction().clone())
    }

    /// Get the metadata of a vertex associated with the given `hash`.
    pub fn get_metadata(&self, hash: &TxHash) -> Option<T> {
        self.vertices.get(hash).map(|vtx| *vtx.value().metadata())
    }

    /// Returns whether the transaction is stored in the Tangle.
    pub fn contains(&self, hash: &TxHash) -> bool {
        self.vertices.contains_key(hash)
    }

    /// Updates the metadata of a particular vertex.
    pub fn update_metadata(&self, hash: &TxHash, metadata: T) {
        self.vertices.get_mut(hash).map(|mut vtx| {
            let vtx = vtx.value_mut();
            *vtx.metadata_mut() = metadata;
        });
    }

    /// Returns the current size of the Tangle.
    pub fn size(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the current number of tips.
    pub fn num_tips(&self) -> usize {
        self.tips.len()
    }

    #[cfg(test)]
    pub(crate) fn num_children(&self, hash: &TxHash) -> usize {
        self.children.get(hash).map_or(0, |r| r.value().len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helper::*;
    use bee_test::transaction::create_random_tx;

    #[test]
    fn new_tangle() {
        let _: Tangle<u8> = Tangle::new();
    }

    #[test]
    fn insert_and_contains() {
        let tangle = Tangle::new();

        let (hash, tx) = create_random_tx();

        let insert1 = tangle.insert(tx.clone(), hash.clone(), ());

        assert!(insert1.is_some());
        assert_eq!(1, tangle.size());
        assert!(tangle.contains(&hash));
        assert_eq!(1, tangle.num_tips());

        let insert2 = tangle.insert(tx, hash, ());

        assert!(insert2.is_none());
        assert_eq!(1, tangle.size());
        assert!(tangle.contains(&hash));
        assert_eq!(1, tangle.num_tips());
    }

    #[test]
    fn count_tips() {
        let (tangle, _, _) = create_test_tangle();

        assert_eq!(1, tangle.num_tips());
    }
}
