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

use bee_crypto::ternary::{sponge::Kerl, Hash};
use bee_protocol::tangle::tangle;
use bee_signing::ternary::wots::WotsPublicKey;
use bee_tangle::{traversal::visit_parents_follow_trunk, TransactionRef};
use bee_transaction::bundled::bundle::incoming::{IncomingRaw, StagedIncomingBundleBuilder};

type IncomingBundleBuilder = StagedIncomingBundleBuilder<TransactionRef, Kerl, WotsPublicKey<Kerl>, IncomingRaw>;

pub(crate) fn load_bundle_builder(hash: &Hash) -> Option<IncomingBundleBuilder> {
    let mut bundle_builder = IncomingBundleBuilder::new();
    let mut done = false;

    visit_parents_follow_trunk(
        tangle(),
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
            bundle_builder.push(transaction.clone());
        },
    );

    match bundle_builder.len() {
        0 => None,
        _ => Some(bundle_builder),
    }
}
