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

use std::{
    sync::atomic::{AtomicU64, Ordering},
    hash::{Hash, Hasher},
    marker::PhantomData,
    cmp::{PartialEq, Eq},
    fmt,
};
use dashmap::DashMap;

pub struct InternKey<T>(u64, PhantomData<T>);

impl<T> Copy for InternKey<T> {}

impl<T> Clone for InternKey<T> {
    fn clone(&self) -> Self { *self }
}

impl<T> PartialEq for InternKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for InternKey<T> {}

impl<T> Hash for InternKey<T> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.0.hash(hasher);
    }
}

impl<T> fmt::Debug for InternKey<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Id({})", self.0)
    }
}

pub struct Interner<T> {
    id_counter: AtomicU64,
    items: DashMap<T, InternKey<T>>,
}

impl<T: Hash + Eq> Default for Interner<T> {
    fn default() -> Self {
        Self {
            id_counter: AtomicU64::new(0),
            items: DashMap::default(),
        }
    }
}

impl<T: Hash + Eq> Interner<T> {
    pub fn intern(&self, item: T) -> InternKey<T> {
        let id_counter = &self.id_counter;

        *self.items
            .entry(item)
            .or_insert_with(|| InternKey(id_counter.fetch_add(1, Ordering::Relaxed), PhantomData))
    }
}
