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

use crate::{state::LedgerState, whiteflag::metadata::WhiteFlagMetadata};

use bee_crypto::ternary::Hash;
use bee_protocol::tangle::MsTangle;
use bee_tangle::helper::load_bundle_builder;
use bee_transaction::{
    bundled::{Bundle, IncomingBundleBuilderError},
    Vertex,
};
use bee_storage::storage::Backend;

use std::collections::HashSet;

const IOTA_SUPPLY: u64 = 2_779_530_283_277_761;

#[derive(Debug)]
pub(crate) enum Error {
    MissingBundle,
    NotATail,
    InvalidBundle(IncomingBundleBuilderError),
}

#[inline]
fn on_bundle<B: Backend>(
    tangle: &MsTangle<B>,
    state: &mut LedgerState,
    hash: &Hash,
    bundle: &Bundle,
    metadata: &mut WhiteFlagMetadata,
) {
    let mut conflicting = false;
    let (mutates, mutations) = bundle.ledger_mutations();

    if !mutates {
        metadata.num_tails_zero_value += 1;
    } else {
        // First pass to look for conflicts.
        for (address, diff) in mutations.iter() {
            let balance = state.get_or_zero(&address) as i64 + diff;

            if balance < 0 || balance.abs() as u64 > IOTA_SUPPLY {
                metadata.num_tails_conflicting += 1;
                conflicting = true;
                break;
            }
        }

        if !conflicting {
            // Second pass to mutate the state.
            for (address, diff) in mutations {
                state.apply_single_diff(address.clone(), diff);
                metadata.diff.apply_single_diff(address, diff);
            }

            metadata.tails_included.push(*hash);
        }
    }

    metadata.num_tails_referenced += 1;

    // TODO this only actually confirm tails
    tangle.update_metadata(&hash, |meta| {
        meta.flags_mut().set_conflicting(conflicting);
        meta.confirm();
        meta.set_milestone_index(metadata.index);
        // TODO Set OTRSI, ...
        // TODO increment metrics confirmed, zero, value and conflict.
    });
}

pub(crate) fn visit_bundles_dfs<B: Backend>(
    tangle: &MsTangle<B>,
    state: &mut LedgerState,
    root: Hash,
    metadata: &mut WhiteFlagMetadata,
) -> Result<(), Error> {
    let mut hashes = vec![root];
    let mut visited = HashSet::new();

    while let Some(hash) = hashes.last() {
        let meta = match tangle.get_metadata(hash) {
            Some(meta) => meta,
            None => {
                if !tangle.is_solid_entry_point(hash) {
                    return Err(Error::MissingBundle);
                } else {
                    visited.insert(*hash);
                    hashes.pop();
                    continue;
                }
            }
        };

        if !meta.flags().is_tail() {
            return Err(Error::NotATail);
        }

        if meta.flags().is_confirmed() {
            visited.insert(*hash);
            hashes.pop();
            continue;
        }

        // TODO pass match to avoid repetitions
        match load_bundle_builder(tangle, hash) {
            Some(builder) => {
                let trunk = builder.trunk();
                let branch = builder.branch();

                if visited.contains(trunk) && visited.contains(branch) {
                    // TODO check valid and strict semantic
                    let bundle = if meta.flags().is_valid() {
                        // We know the bundle is valid so we can safely skip validation rules.
                        unsafe { builder.build() }
                    } else {
                        match builder.validate() {
                            Ok(builder) => {
                                tangle.update_metadata(&hash, |meta| meta.flags_mut().set_valid(true));
                                builder.build()
                            }
                            Err(e) => return Err(Error::InvalidBundle(e)),
                        }
                    };
                    on_bundle(tangle, state, hash, &bundle, metadata);
                    visited.insert(*hash);
                    hashes.pop();
                } else if !visited.contains(trunk) {
                    hashes.push(*trunk);
                } else if !visited.contains(branch) {
                    hashes.push(*branch);
                }
            }
            None => {
                if !tangle.is_solid_entry_point(hash) {
                    return Err(Error::MissingBundle);
                } else {
                    visited.insert(*hash);
                    hashes.pop();
                }
            }
        }
    }

    Ok(())
}
