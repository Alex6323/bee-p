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

use crate::{
    bundled::{Address, BundledTransaction, BundledTransactionField, BundledTransactions},
    Vertex,
};

use bee_crypto::ternary::Hash;

use std::collections::HashMap;

pub struct Bundle<T>(pub(crate) BundledTransactions<T>);

impl<T: AsRef<BundledTransaction>> Bundle<T> {
    // TODO TEST
    pub fn get(&self, index: usize) -> Option<&BundledTransaction> {
        self.0.get(index)
    }

    // TODO TEST
    pub fn len(&self) -> usize {
        self.0.len()
    }

    // TODO TEST
    pub fn hash(&self) -> &Hash {
        // Safe to unwrap because empty bundles can't be built
        self.get(0).unwrap().bundle()
    }

    // TODO TEST
    pub fn tail(&self) -> &BundledTransaction {
        // Safe to unwrap because empty bundles can't be built
        self.get(0).unwrap()
    }

    // TODO TEST
    pub fn head(&self) -> &BundledTransaction {
        // Safe to unwrap because empty bundles can't be built
        self.get(self.len() - 1).unwrap()
    }

    // TODO TEST
    pub fn ledger_mutations(&self) -> (bool, HashMap<Address, i64>) {
        let mut diff = HashMap::new();

        for transaction in self {
            if *transaction.as_ref().value.to_inner() != 0 {
                *diff.entry(transaction.as_ref().address().clone()).or_insert(0) +=
                    *transaction.as_ref().value.to_inner();
            }
        }

        for (_, value) in diff.iter() {
            if *value != 0 {
                return (true, diff);
            }
        }

        (false, diff)
    }
}

impl<T: AsRef<BundledTransaction>> Vertex for Bundle<T> {
    type Hash = Hash;

    // TODO TEST
    fn trunk(&self) -> &Hash {
        self.head().trunk()
    }

    // TODO TEST
    fn branch(&self) -> &Hash {
        self.head().branch()
    }
}

impl<T: AsRef<BundledTransaction>> IntoIterator for Bundle<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    // TODO TEST
    fn into_iter(self) -> Self::IntoIter {
        (self.0).0.into_iter()
    }
}

impl<'a, T: AsRef<BundledTransaction>> IntoIterator for &'a Bundle<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    // TODO TEST
    fn into_iter(self) -> Self::IntoIter {
        (&(self.0).0).iter()
    }
}

impl<T: AsRef<BundledTransaction>> std::ops::Index<usize> for Bundle<T> {
    type Output = BundledTransaction;

    // TODO TEST
    fn index(&self, index: usize) -> &Self::Output {
        // Unwrap because index is expected to panic if out of range
        self.get(index).unwrap()
    }
}

#[cfg(test)]
mod tests {}
