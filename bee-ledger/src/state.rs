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

type InnerState = HashMap<Address, u64>;

#[derive(Default)]
pub struct LedgerState(InnerState);

impl LedgerState {
    /// Creates a new `LedgerState` from its inner type.
    pub fn new(state: InnerState) -> Self {
        state.into()
    }

    /// Gets the balance of an address or zero.
    pub fn get_or_zero(&self, address: &Address) -> u64 {
        self.0.get(address).cloned().unwrap_or(0)
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
}

impl From<InnerState> for LedgerState {
    fn from(state: InnerState) -> Self {
        Self(state)
    }
}
