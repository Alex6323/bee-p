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

use crate::whiteflag::{confirmation::Confirmation, worker::LedgerWorker};

use bee_crypto::ternary::Hash;
use bee_protocol::tangle::tangle;
use bee_tangle::traversal::visit_parents_follow_trunk;
use bee_transaction::{
    bundled::{Bundle, IncomingBundleBuilder, IncomingBundleBuilderError},
    Vertex,
};

use std::collections::HashSet;

const IOTA_SUPPLY: u64 = 2_779_530_283_277_761;

#[derive(Debug)]
pub(crate) enum Error {
    MissingHash,
    InvalidBundle(IncomingBundleBuilderError),
}

fn load_bundle_builder(hash: &Hash) -> Option<IncomingBundleBuilder> {
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
            bundle_builder.push((*(*transaction)).clone());
        },
    );

    match bundle_builder.len() {
        0 => None,
        _ => Some(bundle_builder),
    }
}

impl LedgerWorker {
    #[inline]
    fn on_bundle(&mut self, hash: &Hash, bundle: &Bundle, confirmation: &mut Confirmation) {
        let (mutates, bundle_mutations) = bundle.ledger_mutations();

        confirmation.num_tails_referenced += 1;

        if !mutates {
            confirmation.num_tails_zero_value += 1;
        } else {
            // First pass to look for conflicts.
            for (address, diff) in bundle_mutations.iter() {
                let balance = *self.state.get_or_zero(&address) as i64 + diff;

                if balance < 0 || balance.abs() as u64 > IOTA_SUPPLY {
                    confirmation.num_tails_conflicting += 1;
                    return;
                }
            }

            // Second pass to mutate the state.
            for (address, diff) in bundle_mutations {
                self.state.apply(address.clone(), diff);
                confirmation.diff.apply(address, diff);
            }

            confirmation.tails_included.insert(*hash);
        }
    }

    pub(crate) fn visit_bundles_dfs(&mut self, root: Hash, confirmation: &mut Confirmation) -> Result<(), Error> {
        let mut hashes = vec![root];
        let mut visited = HashSet::new();

        while let Some(hash) = hashes.last() {
            // TODO pass match to avoid repetitions
            match load_bundle_builder(hash) {
                Some(bundle_builder) => {
                    let trunk = bundle_builder.trunk();
                    let branch = bundle_builder.branch();
                    // TODO justify
                    let meta = tangle().get_metadata(hash).unwrap();

                    if visited.contains(trunk) && visited.contains(branch) {
                        let bundle = match bundle_builder.validate() {
                            Ok(builder) => builder.build(),
                            Err(e) => return Err(Error::InvalidBundle(e)),
                        };
                        self.on_bundle(hash, &bundle, confirmation);
                        visited.insert(hash.clone());
                        hashes.pop();
                    } else if !visited.contains(trunk) {
                        if !meta.flags().is_confirmed() {
                            hashes.push(*trunk);
                        }
                    } else if !visited.contains(branch) {
                        if !meta.flags().is_confirmed() {
                            hashes.push(*branch);
                        }
                    }
                }
                None => {
                    if !tangle().is_solid_entry_point(hash) {
                        return Err(Error::MissingHash);
                    } else {
                        visited.insert(hash.clone());
                        hashes.pop();
                    }
                }
            }
        }

        Ok(())
    }
}
