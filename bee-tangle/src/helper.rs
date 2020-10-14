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
#![allow(missing_docs)]

use crate::{
    tangle::{Hooks, Tangle},
    traversal::visit_parents_follow_parent1,
};

use bee_crypto::ternary::Hash;
use bee_transaction::bundled::IncomingBundleBuilder;

pub fn load_bundle_builder<Metadata, H: Hooks<Metadata>>(
    tangle: &Tangle<Metadata, H>,
    hash: &Hash,
) -> Option<IncomingBundleBuilder>
where
    Metadata: Clone + Copy,
{
    let mut bundle_builder = IncomingBundleBuilder::default();
    let mut done = false;

    visit_parents_follow_parent1(
        tangle,
        *hash,
        |transaction, _| {
            if done {
                return false;
            }
            if transaction.index() == transaction.last_index() {
                done = true;
            }
            true
        },
        |_, transaction, _| {
            bundle_builder.push((*(*transaction)).clone());
        },
    );

    match bundle_builder.len() {
        0 => None,
        _ => Some(bundle_builder),
    }
}
