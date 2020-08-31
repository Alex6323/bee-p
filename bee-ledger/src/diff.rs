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

use bee_storage::persistable::Persistable;

use std::collections::HashMap;

type InnerDiff = HashMap<Address, i64>;

#[derive(Default)]
pub struct LedgerDiff(pub(crate) InnerDiff);

impl LedgerDiff {
    pub fn new(diff: InnerDiff) -> Self {
        diff.into()
    }

    pub(crate) fn apply(&mut self, address: Address, diff: i64) {
        self.0.entry(address).and_modify(|d| *d += diff).or_insert(diff);
    }
}

impl Persistable for LedgerDiff {
    fn encode_persistable(&self, buffer: &mut Vec<u8>) {
        self.0.encode_persistable(buffer)
    }
    fn decode_persistable(slice: &[u8]) -> Self {
        LedgerDiff(HashMap::decode_persistable(slice))
    }
}

impl From<InnerDiff> for LedgerDiff {
    fn from(diff: InnerDiff) -> Self {
        Self(diff)
    }
}

impl IntoIterator for LedgerDiff {
    type Item = (Address, i64);
    type IntoIter = std::collections::hash_map::IntoIter<Address, i64>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
