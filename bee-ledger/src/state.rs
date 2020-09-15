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

use crate::diff::LedgerDiff;

use bee_transaction::bundled::Address;

use std::{collections::HashMap, convert::From};

#[derive(Default)]
pub struct LedgerState(HashMap<Address, u64>);

impl LedgerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self(HashMap::with_capacity(cap))
    }

    /// Gets the balance of an address or zero.
    pub fn get_or_zero(&self, address: &Address) -> u64 {
        self.0.get(address).cloned().unwrap_or(0)
    }

    pub fn insert(&mut self, address: Address, balance: u64) -> Option<u64> {
        self.0.insert(address, balance)
    }

    /// Applies a difference to an address.
    pub fn apply_single_diff(&mut self, address: Address, diff: i64) {
        self.0
            .entry(address)
            .and_modify(|d| *d = (*d as i64 + diff) as u64)
            .or_insert(diff as u64);
    }

    pub fn apply_diff(&mut self, diff: LedgerDiff) {
        for (address, value) in diff {
            self.apply_single_diff(address, value);
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Address, &u64)> {
        self.0.iter()
    }
}

impl From<HashMap<Address, u64>> for LedgerState {
    fn from(state: HashMap<Address, u64>) -> Self {
        Self(state)
    }
}

impl IntoIterator for LedgerState {
    type Item = (Address, u64);
    type IntoIter = std::collections::hash_map::IntoIter<Address, u64>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
