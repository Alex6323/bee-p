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

use bee_crypto::ternary::Hash;
use bee_protocol::MilestoneIndex;

/// White flag metadata of a milestone confirmation.
#[derive(Default)]
pub(crate) struct WhiteFlagMetadata {
    /// Index of the confirming milestone.
    pub(crate) index: MilestoneIndex,
    /// Timestamp of the confirming milestone.
    #[allow(dead_code)]
    pub(crate) timestamp: u64,
    /// The number of tails which were referenced by the confirming milestone.
    pub(crate) num_tails_referenced: usize,
    /// The number of tails which were excluded because they were part of a zero or spam value transfer.
    pub(crate) num_tails_zero_value: usize,
    /// The number of tails which were excluded as they were conflicting with the ledger state.
    pub(crate) num_tails_conflicting: usize,
    /// The tails of bundles which mutate the ledger in the order in which they were applied.
    pub(crate) tails_included: Vec<Hash>,
}

impl WhiteFlagMetadata {
    /// Creates a new white flag metadata.
    pub(crate) fn new(index: MilestoneIndex, timestamp: u64) -> WhiteFlagMetadata {
        WhiteFlagMetadata {
            index,
            timestamp,
            ..Self::default()
        }
    }
}
