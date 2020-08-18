// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use crate::{vertex::Vertex, TransactionRef as TxRef};

use bee_crypto::ternary::Hash;
use bee_transaction::{bundled::BundledTransaction as Tx, Vertex as MessageVertex};

use dashmap::{mapref::entry::Entry, DashMap, DashSet};

use std::collections::HashSet;

/// A foundational, thread-safe graph datastructure to represent the IOTA Tangle.
pub struct Tangle<T>
where
    T: Clone + Copy,
{
    pub(crate) vertices: DashMap<Hash, Vertex<T>>,
    pub(crate) children: DashMap<Hash, HashSet<Hash>>,
    pub(crate) tips: DashSet<Hash>,
    // TODO: PriorityQueue with customizable priority for implementing cache eviction strategy
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
    pub fn insert(&self, hash: Hash, transaction: Tx, metadata: T) -> Option<TxRef> {
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
    fn add_child(&self, parent: Hash, child: Hash) {
        match self.children.entry(parent) {
            Entry::Occupied(mut entry) => {
                let children = entry.get_mut();
                children.insert(child);
            }
            Entry::Vacant(entry) => {
                // TODO: find a good value for pre-allocation
                let mut children = HashSet::new();
                children.insert(child);
                entry.insert(children);
            }
        }
    }

    /// Get the data of a vertex associated with the given `hash`.
    pub fn get(&self, hash: &Hash) -> Option<TxRef> {
        self.vertices.get(hash).map(|vtx| vtx.value().transaction().clone())
    }

    /// Returns whether the transaction is stored in the Tangle.
    pub fn contains(&self, hash: &Hash) -> bool {
        self.vertices.contains_key(hash)
    }

    /// Get the metadata of a vertex associated with the given `hash`.
    pub fn get_metadata(&self, hash: &Hash) -> Option<T> {
        self.vertices.get(hash).map(|vtx| *vtx.value().metadata())
    }

    /// Updates the metadata of a particular vertex.
    pub fn set_metadata(&self, hash: &Hash, metadata: T) {
        self.vertices.get_mut(hash).map(|mut vtx| {
            *vtx.value_mut().metadata_mut() = metadata;
        });
    }

    /// Updates the metadata of a vertex.
    pub fn update_metadata<Update>(&self, hash: &Hash, update: Update)
    where
        Update: Fn(&mut T),
    {
        self.vertices
            .get_mut(hash)
            .map(|mut vtx| update(vtx.value_mut().metadata_mut()));
    }

    /// Returns the number of transactions in the Tangle.
    pub fn len(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the children of a vertex.
    pub fn get_children(&self, hash: &Hash) -> HashSet<Hash> {
        let num_children = self.num_children(hash);
        let mut hashes = HashSet::with_capacity(num_children);

        self.children.get(hash).map(|c| {
            for child in c.value() {
                hashes.insert(*child);
            }
        });

        hashes
    }

    /// Returns the current number of tips.
    pub fn num_tips(&self) -> usize {
        self.tips.len()
    }

    /// Returns all tips of the tangle.
    pub fn tips(&self) -> HashSet<Hash> {
        let mut ret = HashSet::new();
        for tip in self.tips.iter() {
            ret.insert(*tip);
        }
        ret
    }

    /// Returns the number of children of a vertex.
    pub fn num_children(&self, hash: &Hash) -> usize {
        self.children.get(hash).map_or(0, |r| r.value().len())
    }

    #[cfg(test)]
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.children.clear();
        self.tips.clear();
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

        let insert1 = tangle.insert(hash.clone(), tx.clone(), ());

        assert!(insert1.is_some());
        assert_eq!(1, tangle.len());
        assert!(tangle.contains(&hash));
        assert_eq!(1, tangle.num_tips());

        let insert2 = tangle.insert(hash, tx, ());

        assert!(insert2.is_none());
        assert_eq!(1, tangle.len());
        assert!(tangle.contains(&hash));
        assert_eq!(1, tangle.num_tips());
    }
}
