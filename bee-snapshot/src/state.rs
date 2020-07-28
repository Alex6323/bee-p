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

use bee_transaction::bundled::Address;

use std::collections::HashMap;

// TODO Abstract balances

#[derive(Default)]
pub struct SnapshotState {
    pub(crate) balances: HashMap<Address, u64>,
}

impl SnapshotState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            balances: HashMap::with_capacity(capacity),
        }
    }

    pub fn insert(&mut self, address: Address, balance: u64) -> Option<u64> {
        self.balances.insert(address, balance)
    }

    pub fn remove(&mut self, address: &Address) -> Option<u64> {
        self.balances.remove(address)
    }

    pub fn get(&self, address: &Address) -> Option<&u64> {
        self.balances.get(address)
    }

    pub fn len(&self) -> usize {
        self.balances.len()
    }

    pub fn balances(&self) -> &HashMap<Address, u64> {
        &self.balances
    }

    pub fn into_balances(self) -> HashMap<Address, u64> {
        self.balances
    }
}
