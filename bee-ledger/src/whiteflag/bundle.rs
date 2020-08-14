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
use bee_protocol::tangle::tangle;
use bee_tangle::traversal::visit_parents_follow_trunk;
use bee_transaction::bundled::BundledTransactionField;

pub(crate) enum Error {
    IncompleteBundle,
}

pub(crate) fn load_bundle(hash: &Hash) -> Result<(), Error> {
    let mut done = false;

    visit_parents_follow_trunk(
        tangle(),
        *hash,
        |tx, _| {
            if done {
                return true;
            }
            if tx.index().to_inner() > tx.last_index().to_inner() {
                done = true;
            }
            done
        },
        |hash, tx, meta| {
            println!(
                "{:?}",
                hash.iter_trytes().map(|trit| char::from(trit)).collect::<String>()
            );
        },
    );

    Ok(())
}
