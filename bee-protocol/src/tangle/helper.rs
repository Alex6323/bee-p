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

use bee_tangle::{traversal, Hooks, Tangle, TransactionRef};

use bee_crypto::ternary::Hash;

pub(crate) fn find_tail_of_bundle<H: Hooks<TransactionMetadata>>(
    tangle: &Tangle<TransactionMetadata, H>,
    root: Hash,
) -> Option<Hash> {
    let mut tail = None;
    let mut bundle = None;

    traversal::visit_children_follow_trunk(
        tangle,
        root,
        |tx, _| {
            if bundle.is_none() {
                bundle.replace(*tx.bundle());
            }

            bundle.as_ref().unwrap() == tx.bundle()
        },
        |hash, tx, _| {
            if tx.is_tail() {
                tail.replace(*hash);
            }
        },
    );

    tail
}

pub fn on_all_tails<Apply: FnMut(&Hash, &TransactionRef, &TransactionMetadata), H: Hooks<TransactionMetadata>>(
    tangle: &Tangle<TransactionMetadata, H>,
    root: Hash,
    apply: Apply,
) {
    traversal::visit_parents_depth_first(
        tangle,
        root,
        |_, _, metadata| !metadata.flags().is_tail(),
        |_, _, _| {},
        apply,
        |_| {},
    );
}
