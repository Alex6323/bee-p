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

use crate::tangle::TransactionMetadata;

use bee_tangle::{traversal, Tangle};

use bee_crypto::ternary::Hash;

pub(crate) fn find_tail_of_bundle(
    tangle: &Tangle<TransactionMetadata>,
    root: Hash,
    bundle_hash: &Hash,
) -> Option<Hash> {
    let mut tail = None;

    traversal::visit_children_follow_trunk(
        tangle,
        root,
        |tx, _| tx.bundle() == bundle_hash,
        |tx_hash, tx, _| {
            if tx.is_tail() {
                tail.replace(*tx_hash);
            }
        },
    );

    tail
}
