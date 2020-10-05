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
use lru::LruCache;

use std::{
    collections::HashSet,
    sync::{RwLock, atomic::{AtomicU64, Ordering}},
    marker::PhantomData,
};

const CACHE_LEN: usize = 65536;

pub trait Hooks<T> {
    type Error;

    fn get(&self, hash: &Hash) -> Result<(Tx, T), Self::Error>;
    fn insert(&self, hash: Hash, tx: Tx, metadata: T) -> Result<(), Self::Error>;
}

pub struct NullHooks<T>(PhantomData<T>);

impl<T> Default for NullHooks<T> {
    fn default() -> Self { Self(PhantomData) }
}

impl<T> Hooks<T> for NullHooks<T> {
    type Error = ();

    fn get(&self, hash: &Hash) -> Result<(Tx, T), Self::Error> {
        Err(())
    }

    fn insert(&self, hash: Hash, tx: Tx, metadata: T) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// A foundational, thread-safe graph datastructure to represent the IOTA Tangle.
pub struct Tangle<T, H = NullHooks<T>>
where
    T: Clone + Copy,
{
    pub(crate) vertices: DashMap<Hash, Vertex<T>>,
    pub(crate) children: DashMap<Hash, HashSet<Hash>>,
    pub(crate) tips: DashSet<Hash>,

    pub(crate) cache_counter: AtomicU64,
    pub(crate) cache_queue: RwLock<LruCache<Hash, u64>>,

    pub(crate) hooks: H,
}

impl<T, H: Hooks<T>> Default for Tangle<T, H>
where
    T: Clone + Copy,
    H: Default,
{
    fn default() -> Self {
        Self::new(H::default())
    }
}

impl<T, H: Hooks<T>> Tangle<T, H>
where
    T: Clone + Copy,
{
    /// Creates a new Tangle.
    pub fn new(hooks: H) -> Self {
        Self {
            vertices: DashMap::new(),
            children: DashMap::new(),
            tips: DashSet::new(),

            cache_counter: AtomicU64::new(0),
            cache_queue: RwLock::new(LruCache::new(CACHE_LEN + 1)),

            hooks,
        }
    }

    /// Create a new tangle with the given capacity.
    pub fn with_capacity(self, cap: usize) -> Self {
        Self {
            cache_queue: RwLock::new(LruCache::new(cap + 1)),
            ..self
        }
    }

    /// Inserts a transaction, and returns a thread-safe reference to it in case it didn't already exist.
    pub async fn insert(&self, hash: Hash, transaction: Tx, metadata: T) -> Option<TxRef> {
        println!("Insert!");

        let r = match self.vertices.entry(hash) {
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

                // Insert cache queue entry to track eviction priority
                self.cache_queue.write().unwrap().put(hash.clone(), self.generate_cache_index());

                Some(tx)
            }
        };

        self.perform_eviction();

        r
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
    pub async fn get(&self, hash: &Hash) -> Option<TxRef> {
        self.pull_transaction(hash).await;

        self.vertices
            .get(hash)
            .map(|vtx| {
                let mut cache_queue = self.cache_queue.write().unwrap();
                // Update hash priority
                let entry = cache_queue.get_mut(hash);
                let entry = if entry.is_none() {
                    cache_queue.put(hash.clone(), 0);
                    cache_queue.get_mut(hash)
                } else {
                    entry
                };
                *entry.unwrap() = self.generate_cache_index();

                vtx.value().transaction().clone()
            })
    }

    /// Returns whether the transaction is stored in the Tangle.
    pub async fn contains(&self, hash: &Hash) -> bool {
        self.vertices.contains_key(hash) || self.pull_transaction(hash).await
    }

    /// Get the metadata of a vertex associated with the given `hash`.
    pub fn get_metadata(&self, hash: &Hash) -> Option<T> {
        self.vertices.get(hash).map(|vtx| *vtx.value().metadata())
    }

    /// Updates the metadata of a particular vertex.
    pub fn set_metadata(&self, hash: &Hash, metadata: T) {
        if let Some(mut vtx) = self.vertices.get_mut(hash) {
            *vtx.value_mut().metadata_mut() = metadata;
        }
    }

    /// Updates the metadata of a vertex.
    pub fn update_metadata<Update>(&self, hash: &Hash, mut update: Update)
    where
        Update: FnMut(&mut T),
    {
        if let Some(mut vtx) = self.vertices.get_mut(hash) {
            update(vtx.value_mut().metadata_mut())
        }
    }

    /// Returns the number of transactions in the Tangle.
    pub fn len(&self) -> usize {
        self.vertices.len()
    }

    /// Checks if the tangle is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the children of a vertex.
    pub fn get_children(&self, hash: &Hash) -> HashSet<Hash> {
        if let Some(c) = self.children.get(hash) {
            c.value().clone()
        } else {
            HashSet::new()
        }
    }

    /// Returns the current number of tips.
    pub fn num_tips(&self) -> usize {
        self.tips.len()
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

    // Attempts to pull the transaction from the storage, returns true if successful.
    async fn pull_transaction(&self, hash: &Hash) -> bool {
        // If the tangle already contains the tx, do no more work
        if self.vertices.contains_key(hash) {
            true
        } else {
            false
            //todo!()
        }
    }

    fn generate_cache_index(&self) -> u64 {
        self.cache_counter.fetch_add(1, Ordering::Relaxed)
    }

    fn perform_eviction(&self) {
        let mut cache = self.cache_queue.write().unwrap();

        assert_eq!(cache.len(), self.len());

        if cache.len() == cache.cap() {
            let (hash, _) = cache.pop_lru().expect("Cache capacity is zero");

            self.vertices.remove(&hash).expect("Expected vertex entry to exist");
            self.children.remove(&hash);
            self.tips.remove(&hash);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bee_test::transaction::create_random_tx;
    use pollster::block_on;

    #[test]
    fn new_tangle() {
        let _: Tangle<u8> = Tangle::new();
    }

    #[test]
    fn insert_and_contains() {
        let tangle = Tangle::new();

        let (hash, tx) = create_random_tx();

        let insert1 = block_on(tangle.insert(hash.clone(), tx.clone(), ()));

        assert!(insert1.is_some());
        assert_eq!(1, tangle.len());
        assert!(block_on(tangle.contains(&hash)));
        assert_eq!(1, tangle.num_tips());

        let insert2 = block_on(tangle.insert(hash, tx, ()));

        assert!(insert2.is_none());
        assert_eq!(1, tangle.len());
        assert!(block_on(tangle.contains(&hash)));
        assert_eq!(1, tangle.num_tips());
    }

    #[test]
    fn eviction_cap() {
        let tangle = Tangle::default().with_capacity(5);

        let txs = (0..10)
            .map(|_| create_random_tx())
            .collect::<Vec<_>>();

        for (hash, tx) in txs.iter() {
            let _ = block_on(tangle.insert(hash.clone(), tx.clone(), ()));
        }

        assert_eq!(tangle.len(), 5);
    }

    #[test]
    fn eviction_update() {
        let tangle = Tangle::default().with_capacity(5);

        let txs = (0..8)
            .map(|_| create_random_tx())
            .collect::<Vec<_>>();

        for (hash, tx) in txs.iter().take(4) {
            let _ = block_on(tangle.insert(hash.clone(), tx.clone(), ()));
        }

        assert!(block_on(tangle.get(&txs[0].0)).is_some());

        for (hash, tx) in txs.iter().skip(4) {
            let _ = block_on(tangle.insert(hash.clone(), tx.clone(), ()));
        }

        assert!(block_on(tangle.contains(&txs[0].0)));

        for entry in tangle.vertices.iter() {
            assert!(entry.key() == &txs[0].0 || txs[4..].iter().find(|(h, _)| entry.key() == h).is_some());
        }
    }
}
