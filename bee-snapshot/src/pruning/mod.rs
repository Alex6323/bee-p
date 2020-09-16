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

use crate::{
    config::SnapshotConfig,
    constants::{ADDITIONAL_PRUNING_THRESHOLD, SOLID_ENTRY_POINT_CHECK_THRESHOLD_PAST},
    local::LocalSnapshotMetadata,
};

use bee_crypto::ternary::Hash;
use bee_protocol::{
    tangle::{helper, tangle},
    MilestoneIndex,
};
use bee_tangle::traversal;

use dashmap::DashMap;

use log::{error, info, warn};

#[derive(Debug)]
pub enum Error {
    NotEnoughHistory,
    NoPruningNeeded,
    MilestoneNotFoundInTangle,
    SolidEntryPointNotConfirmed,
    MetadataNotFound,
    MilestoneTransactionIsNotTail,
}

/// Checks whether any direct approver of the given transaction was confirmed by a
/// milestone which is above the target milestone.
pub fn is_solid_entry_point(hash: &Hash) -> Result<bool, Error> {
    // Check if there is any child of the transaction is confirmed by newer milestones
    let milestone_index;
    if let Some(metadata) = tangle().get_metadata(hash) {
        milestone_index = metadata.milestone_index;
    } else {
        error!("Metadada for hash {:?} is not found in Tangle.", hash);
        return Err(Error::MetadataNotFound);
    }
    let mut is_solid = false;
    traversal::visit_children_follow_trunk(
        tangle(),
        *hash,
        |_tx, metadata| {
            if is_solid {
                return false;
            }
            // is_solid: one of the current tx's approver is confirmed by a newer milestone_index
            is_solid = metadata.flags.is_confirmed() && metadata.milestone_index > milestone_index;
            true
        },
        |_hash, _tx, _metadata| {},
    );
    Ok(is_solid)
}

// TODO testing
pub fn get_new_solid_entry_points(target_index: MilestoneIndex) -> Result<DashMap<Hash, MilestoneIndex>, Error> {
    let solid_entry_points = DashMap::<Hash, MilestoneIndex>::new();
    for index in *target_index - SOLID_ENTRY_POINT_CHECK_THRESHOLD_PAST..*target_index {
        let milestone_hash;

        // Get the milestone tail hash
        // NOTE this milestone hash must be tail hash
        if let Some(hash) = tangle().get_milestone_hash(MilestoneIndex(index)) {
            // NOTE Actually we don't really need the tail, and only need one of the milestone tx.
            //      In gohornet, we start from the tail milestone tx.
            milestone_hash = hash;
        // if !tangle().get_metadata(&hash).unwrap().flags.is_tail() {
        //     error!("Milestone w/ hash {} is not tail.", hash);
        //     return Err(Error::MilestoneTransactionIsNotTail);
        // } else {
        //     milestone_hash = hash;
        // }
        } else {
            error!("Milestone index {} is not found in Tangle.", index);
            return Err(Error::MilestoneNotFoundInTangle);
        }

        // Get all the approvees confirmed by the milestone tail
        traversal::visit_parents_depth_first(
            tangle(),
            milestone_hash,
            |_hash, _tx, metadata| *metadata.milestone_index() >= index,
            |hash, _tx, metadata| {
                if metadata.flags.is_confirmed() && is_solid_entry_point(&hash).unwrap() {
                    // Find all tails
                    helper::on_all_tails(tangle(), *hash, |hash, _tx, metadata| {
                        solid_entry_points.insert(hash.clone(), metadata.milestone_index);
                    });
                }
            },
            |_hash, _tx, _metadata| {},
            |_hash| {},
        );
    }
    Ok(solid_entry_points)
}

// TODO get the unconfirmed trnsactions in the database
pub fn get_unconfirmed_transactions(target_index: &MilestoneIndex) -> Vec<Hash> {
    // NOTE If there is no specific struct for storing th unconfirmed transaction,
    //      then we need to traverse the whole tangle to get the unconfirmed transactions (SLOW)!
    // TODO traverse the whole tangle through the approvers from solid entry points
    unimplemented!()
}

// TODO remove the unconfirmed transactions in the database
pub fn prune_unconfirmed_transactions(purning_milestone_index: &MilestoneIndex) -> u32 {
    unimplemented!()
}

// TODO remove the confirmed transactions in the database
pub fn prune_transactions(hashes: Vec<Hash>) -> u32 {
    unimplemented!()
}

// TODO prunes the milestone metadata and the ledger diffs from the database for the given milestone
pub fn prune_milestone(milestone_index: MilestoneIndex) {
    // delete ledger_diff for milestone with milestone_index
    // delete milestone storage (if we have this) for milestone with milestone_index
    unimplemented!()
}

// NOTE we don't prune cache, but only prune the database
pub fn prune_database(mut target_index: MilestoneIndex) -> Result<(), Error> {
    // NOTE the pruning happens after `createLocalSnapshot`, so the metadata should provide the latest index
    if *tangle().get_entry_point_index() < SOLID_ENTRY_POINT_CHECK_THRESHOLD_PAST + ADDITIONAL_PRUNING_THRESHOLD + 1 {
        error!(
            "Not enough histroy for pruning! minimum index: {}, target index: {}",
            SOLID_ENTRY_POINT_CHECK_THRESHOLD_PAST + ADDITIONAL_PRUNING_THRESHOLD + 1,
            *target_index
        );
        return Err(Error::NotEnoughHistory);
    }

    // TODO change the type of `LocalSnapshotMetadata.index` to MilestoneIndex?
    let target_index_max = MilestoneIndex(
        *tangle().get_snapshot_index() - SOLID_ENTRY_POINT_CHECK_THRESHOLD_PAST - ADDITIONAL_PRUNING_THRESHOLD - 1,
    );
    if target_index > target_index_max {
        target_index = target_index_max;
    }

    if tangle().get_pruning_index() >= target_index {
        error!(
            "No pruning needed with pruning index: {:?} and target_index: {:?}",
            tangle().get_pruning_index(),
            target_index
        );
        return Err(Error::NoPruningNeeded);
    }

    if *tangle().get_latest_solid_milestone_index() + ADDITIONAL_PRUNING_THRESHOLD + 1 > *target_index {
        // we prune in "ADDITIONAL_PRUNING_THRESHOLD" steps to recalculate the solid_entry_points
        error!(
            "Not enough history! minimum index: {:?} should be <= target_index: {:?}",
            tangle().get_latest_solid_milestone_index()
                + MilestoneIndex(ADDITIONAL_PRUNING_THRESHOLD)
                + MilestoneIndex(1),
            target_index
        );
        return Err(Error::NotEnoughHistory);
    }

    // Update the solid entry points in the static MsTangle
    let new_solid_entry_points = get_new_solid_entry_points(target_index)?;

    // TODO clear the solid_entry_points in the static MsTangle
    // tangle().clear_solid_entry_points()

    // TODO update the whole solid_entry_points in the static MsTangle w/o looping
    for (hash, milestone_index) in new_solid_entry_points.into_iter() {
        tangle().add_solid_entry_point(hash, milestone_index);
    }

    tangle().update_entry_point_index(target_index);

    prune_unconfirmed_transactions(&tangle().get_pruning_index());

    // Iterate through all milestones that have to be pruned
    for milestone_index in *tangle().get_pruning_index() + 1..*target_index + 1 {
        info!("Pruning milestone {}...", milestone_index);

        let pruned_unconfirmed_transactions_count = prune_unconfirmed_transactions(&MilestoneIndex(milestone_index));

        // Get the milestone tail hash
        // NOTE this milestone hash must be tail hash
        let milestone_hash;
        if let Some(hash) = tangle().get_milestone_hash(MilestoneIndex(milestone_index)) {
            // NOTE Actually we don't really need the tail, and only need one of the milestone tx.
            //      In gohornet, we start from the tail milestone tx.
            milestone_hash = hash;
        // if !tangle().get_metadata(&hash).unwrap().flags.is_tail() {
        //     error!("Milestone w/ hash {} is not tail.", hash);
        //     return Err(Error::MilestoneTransactionIsNotTail);
        // } else {
        //     milestone_hash = hash;
        // }
        } else {
            warn!("Pruning milestone {} failed! Milestone not found!", milestone_index);
            continue;
        }

        let mut transactions_to_prune: Vec<Hash> = Vec::new();

        // Traverse the approvees of the milestone transaction
        // Get all the approvees confirmed by the milestone tail
        // Ignore Tx that were confirmed by older milestones
        traversal::visit_parents_depth_first(
            tangle(),
            milestone_hash,
            |_hash, _tx, _metadata| {
                // NOTE everything that was referenced by that milestone can be pruned
                //      (even transactions of older milestones)
                true
            },
            |hash, _tx, _metadata| transactions_to_prune.push(hash.clone()),
            |_hash, _tx, _metadata| {},
            |_hash| {},
        );

        // NOTE The metadata of solid entry points can be deleted from the database,
        //      because we only need the hashes of them, and don't have to trace their parents.
        let transactions_to_prune_count = transactions_to_prune.len();
        let pruned_transactions_count = prune_transactions(transactions_to_prune);

        prune_milestone(MilestoneIndex(milestone_index));

        tangle().update_pruning_index(MilestoneIndex(milestone_index));
        info!(
            "Pruning milestone {}. Pruned {}/{} transactions. Pruned {} unconfirmed transactions",
            milestone_index,
            pruned_transactions_count,
            transactions_to_prune_count,
            pruned_unconfirmed_transactions_count
        );
        // TODO trigger pruning milestone index changed event
    }
    Ok(())
}
