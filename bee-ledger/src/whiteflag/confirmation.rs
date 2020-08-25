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

use bee_crypto::ternary::Hash;

use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct Confirmation {
    pub(crate) diff: LedgerDiff,
    // /// The number of tails which were referenced by the milestone.
    // pub(crate) num_tails_referenced: usize,
    // TODO temporary until traversals can mutate the meta
    pub(crate) tails_referenced: HashSet<Hash>,
    /// The number of tails which were excluded because they were part of a zero or spam value transfer.
    pub(crate) num_tails_zero_value: usize,
    /// The number of tails which were excluded as they were conflicting with the ledger state.
    pub(crate) num_tails_conflicting: usize,
    /// The tails of bundles which mutate the ledger in the order in which they were applied.
    pub(crate) tails_included: HashSet<Hash>,
}

impl Confirmation {
    pub(crate) fn new() -> Confirmation {
        Self::default()
    }
}
