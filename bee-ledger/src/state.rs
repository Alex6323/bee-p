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

#[derive(Default)]
pub(crate) struct LedgerState(pub(crate) HashMap<Address, u64>);

impl LedgerState {
    pub(crate) fn get_or_zero(&self, address: &Address) -> &u64 {
        self.0.get(address).unwrap_or(&0)
    }

    pub(crate) fn apply(&mut self, address: Address, diff: i64) {
        self.0
            .entry(address)
            .and_modify(|d| *d = (*d as i64 + diff) as u64)
            .or_insert(diff as u64);
    }
}
