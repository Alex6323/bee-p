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

use bee_protocol::tangle::tangle;

use log::error;

const SOLID_ENTRY_POINT_CHECK_THRESHOLD_PAST: u32 = 50;
const ADDITIONAL_PRUNING_THRESHOLD: u32 = 50;

pub enum Error {
    NotEnoughHistory,
    NoPruningNeeded,
}

// TODO get the new solid entry points from MsTangle
pub fn get_new_solid_entry_points(_target_index: u32) {
    unimplemented!()
}

// TODO do we rename it to be prune cache?
pub fn prune_database(
    pruning_config: &PruningConfig,
    _local_snapshot_config: &LocalSnapshotConfig,
    local_snapshot_metadata: &LocalSnapshotMetadata,
    mut target_index: u32,
) -> Result<(), Error> {
    // TODO move this checking before enterning this function
    if !pruning_config.enabled() {
        return Ok(());
    }

    if local_snapshot_metadata.index() < ADDITIONAL_PRUNING_THRESHOLD + ADDITIONAL_PRUNING_THRESHOLD + 1 {
        error!("Not enough histroy for pruning.");
        return Err(Error::NotEnoughHistory);
    }

    let target_index_max: u32 =
        local_snapshot_metadata.index() - SOLID_ENTRY_POINT_CHECK_THRESHOLD_PAST - ADDITIONAL_PRUNING_THRESHOLD - 1;
    if target_index > target_index_max {
        target_index = target_index_max;
    }

    // TODO add pruning_index in the tangle or somewhere, which is static and stateful
    // if tangle().pruning_index() >= target_index {
    //     Err(Error::NoPruningNeeded)
    // }

    // TODO add entry_point_index in the tangle or somewhere, which is static and stateful
    // if tangle().entry_point_index() + ADDITIONAL_PRUNING_THRESHOLD + 1 > target_index {
    //     Err(Error::NotEnoughHistory)
    // }

    // TODO update the solid entry points in the static MsTangle

    // TODO prune unconfirmed transaction with milestone tangle().entry_point_index() in database

    // TODO iterate through all milestones that have to be pruned
    //      range: (milestone tangle().entry_point_index() + 1 ~ target_index)

    // TODO record the pruned transaction count for logging ?

    // TODO prune the milestone by milestone_index

    // TODO update the tangle().pruning_index()

    Ok(())
}
