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

mod config;

pub use config::{PruningConfig, PruningConfigBuilder};

use crate::local::{LocalSnapshotConfig, LocalSnapshotMetadata};

use bee_crypto::ternary::Hash;
use bee_protocol::{tangle::tangle, MilestoneIndex};
use bee_tangle::traversal;

use std::collections::HashSet;

use dashmap::DashMap;

use log::{error, info};

const SOLID_ENTRY_POINT_CHECK_THRESHOLD_PAST: MilestoneIndex = MilestoneIndex(50);
const ADDITIONAL_PRUNING_THRESHOLD: MilestoneIndex = MilestoneIndex(50);

pub enum Error {
    NotEnoughHistory,
    NoPruningNeeded,
    MilestoneNotFoundInTangle,
    SolidEntryPointNotConfirmed,
    MetadataNotFound,
    MilestoneTransactionIsNotTail,
}

// TODO
pub fn is_solid_entry_point(hash: &Hash) -> bool {
    unimplemented!()
}

// TODO we need the following in the traversal mod to enable us collect the tails
//      or we enhance the visit_parents_depth_first function
// pub fn visit_parents_depth_first_with_leaf_apply<Metadata, Match, Apply, LeafApply, ElseApply>(
//     tangle: &Tangle<Metadata>,
//     root: Hash,
//     matches: Match,
//     mut apply: Apply,
//     mut leaf_apply: LeafApply,
//     mut else_apply: ElseApply,
// ) where
//     Metadata: Clone + Copy,
//     Match: Fn(&Hash, &TxRef, &Metadata) -> bool,
//     Apply: FnMut(&Hash, &TxRef, &Metadata),
//     LeafApply: FnMut(&Hash, &TxRef, &Metadata),
//     ElseApply: FnMut(&Hash),
// {
//     let mut parents = Vec::new();
//     let mut visited = HashSet::new();

//     parents.push(root);

//     while let Some(hash) = parents.pop() {
//         if !visited.contains(&hash) {
//             match tangle.vertices.get(&hash) {
//                 Some(vtx) => {
//                     let vtx = vtx.value();

//                     if matches(&hash, vtx.transaction(), vtx.metadata()) {
//                         apply(&hash, vtx.transaction(), vtx.metadata());

//                         parents.push(*vtx.trunk());
//                         parents.push(*vtx.branch());
//                     } else {
//                         leaf_apply(&hash, vtx.transaction(), vtx.metadata());
//                     }
//                 }
//                 None => {
//                     else_apply(&hash);
//                 }
//             }
//             visited.insert(hash);
//         }
//     }
// }

// TODO testing
pub fn get_new_solid_entry_points(target_index: MilestoneIndex) -> Result<DashMap<Hash, MilestoneIndex>, Error> {
    let mut solid_entry_points = DashMap::<Hash, MilestoneIndex>::new();
    for index in *target_index - *SOLID_ENTRY_POINT_CHECK_THRESHOLD_PAST..*target_index {
        let mut milestone_tail_hash = Hash::zeros();
        let mut confirmed_transaction_hashes: Vec<Hash> = Vec::new();

        // Get the milestone tail hash
        // NOTE this milestone hash must be tail hash
        match tangle().get_milestone_hash(MilestoneIndex(index)) {
            None => {
                error!("Milestone index {} is not found in Tangle.", index);
                return Err(Error::MilestoneNotFoundInTangle);
            }
            Some(hash) => {
                if !tangle().get_metadata(&hash).unwrap().flags.is_tail() {
                    error!("Milestone w/ hash {} is not tail.", hash);
                    return Err(Error::MilestoneTransactionIsNotTail);
                } else {
                    milestone_tail_hash = hash;
                }
            }
        }
        // Get all the approvees confirmed by the milestone tail
        traversal::visit_parents_depth_first(
            tangle(),
            milestone_tail_hash,
            |hash, _tx, metadata| {
                ((metadata.flags.is_confirmed() && *metadata.milestone_index() >= index)
                    || (!metadata.flags.is_confirmed()))
                    && !tangle().is_solid_entry_point(hash)
            },
            |hash, _tx, metadata| {
                if metadata.flags.is_confirmed() {
                    confirmed_transaction_hashes.push(hash.clone())
                }
            },
            |_hash| {},
        );

        for approvee in confirmed_transaction_hashes {
            // TODO is_solid_entry_point() checks whether any direct approver of the given
            //      transaction was confirmed by a milestone which is above the target milestone.
            if is_solid_entry_point(&approvee) {
                // Find all tails
                let mut tail_hashes: Vec<Hash> = Vec::new();

                // Get all the tails
                // traversal::visit_parents_depth_first_with_leaf_apply(
                //     tangle(),
                //     root,
                //     |_hash, _tx, metadata| metadata.flags.is_tail(),
                //     |_hash, _tx, _metadata| {},
                //     |hash, _tx, _metadata| tail_hashes.push(hash.clone()),
                //     |_hash| {},
                // );

                for tail_hash in tail_hashes {
                    match tangle().get_metadata(&tail_hash) {
                        Some(metadata) => {
                            if metadata.flags.is_confirmed() {
                                solid_entry_points.insert(tail_hash.clone(), metadata.milestone_index);
                            } else {
                                error!("Solid entry point for hash {:?} is not confirmed.", tail_hash);
                                return Err(Error::SolidEntryPointNotConfirmed);
                            }
                        }
                        None => {
                            error!("Metadada for hash {:?} is not found in Tangle.", tail_hash);
                            return Err(Error::MetadataNotFound);
                        }
                    }
                }
            }
        }
    }
    Ok(solid_entry_points)
}

pub fn get_unconfirmed_transactions(target_index: &MilestoneIndex) {
    // NOTE w/o cache, need to traverse the whole tangle to get the unconfirmed transactions!
    // TODO traverse the whole tangle through the approvers from solid entry points
    unimplemented!()
}

// TODO remove the transactions in the database
pub fn prune_unconfirmed_transactions(purning_milestone_index: &MilestoneIndex) {
    unimplemented!()
}

// NOTE we don't prune cache, but only prune the database
pub fn prune_database(
    pruning_config: &PruningConfig,
    _local_snapshot_config: &LocalSnapshotConfig,
    local_snapshot_metadata: &LocalSnapshotMetadata,
    mut target_index: MilestoneIndex,
) -> Result<(), Error> {
    // TODO move this checking before enterning this function
    if !pruning_config.enabled() {
        return Ok(());
    }

    // NOTE the pruning happens after `createLocalSnapshot`, so the metadata should provide the latest index
    // TODO change the type of `LocalSnapshotMetadata.index` to MilestoneIndex?
    if local_snapshot_metadata.index() < *ADDITIONAL_PRUNING_THRESHOLD + *ADDITIONAL_PRUNING_THRESHOLD + 1 {
        error!("Not enough histroy for pruning.");
        return Err(Error::NotEnoughHistory);
    }

    // TODO change the type of `LocalSnapshotMetadata.index` to MilestoneIndex?
    let target_index_max = MilestoneIndex(
        local_snapshot_metadata.index() - *SOLID_ENTRY_POINT_CHECK_THRESHOLD_PAST - *ADDITIONAL_PRUNING_THRESHOLD - 1,
    );
    if target_index > target_index_max {
        target_index = target_index_max;
    }

    if tangle().get_pruning_index() >= target_index {
        error!(
            "No puruning needed with purning index: {:?} and target_index: {:?}",
            tangle().get_pruning_index(),
            target_index
        );
        return Err(Error::NoPruningNeeded);
    }

    if tangle().get_latest_solid_milestone_index() + ADDITIONAL_PRUNING_THRESHOLD + MilestoneIndex(1) > target_index {
        // we prune in "ADDITIONAL_PRUNING_THRESHOLD" steps to recalculate the solid_entry_points
        error!(
            "Not enough history! minimum index: {:?} should be <= target_index: {:?}",
            tangle().get_latest_solid_milestone_index() + ADDITIONAL_PRUNING_THRESHOLD + MilestoneIndex(1),
            target_index
        );
        return Err(Error::NotEnoughHistory);
    }

    // Update the solid entry points in the static MsTangle
    let new_solid_entry_points = get_new_solid_entry_points(target_index)?;

    // TODO clear the solid_entry_points in the static MsTangle
    // tangle().solid_entry_points().clear();

    // TODO update the whole solid_entry_points in the static MsTangle w/o looping
    for (hash, milestone_index) in new_solid_entry_points.into_iter() {
        tangle().add_solid_entry_point(hash, milestone_index);
    }

    tangle().update_latest_solid_milestone_index(target_index);

    // TODO prune unconfirmed transaction with milestone tangle().entry_point_index() in database

    // TODO iterate through all milestones that have to be pruned
    //      range: (milestone tangle().entry_point_index() + 1 ~ target_index)

    // TODO in record the pruned transaction count for logging ?

    // TODO prune the milestone by milestone_index

    // TODO update the tangle().pruning_index()
    // tangle().set_pruning_milestone_index() = target_index

    Ok(())
}
