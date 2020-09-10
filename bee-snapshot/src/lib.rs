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

pub mod config;
pub mod constants;
pub mod event;
pub mod global;
pub mod local;
pub mod pruning;
pub mod worker;

use bee_common::shutdown::Shutdown;
use bee_common_ext::event::Bus;
use bee_protocol::{event::LastSolidMilestoneChanged, MilestoneIndex};

use std::{path::Path, sync::Arc};

fn on_last_solid_milestone_changed(_last_solid_milestone: &LastSolidMilestoneChanged) {
    // TODO send event to worker
}

pub fn init(config: &config::SnapshotConfig, bus: Arc<Bus<'static>>, _shutdown: &mut Shutdown) {
    // snapshotDepth = milestone.Index(config.NodeConfig.GetInt(config.CfgLocalSnapshotsDepth))
    // if snapshotDepth < SolidEntryPointCheckThresholdFuture {
    // 	log.Warnf("Parameter '%s' is too small (%d). Value was changed to %d", config.CfgLocalSnapshotsDepth, snapshotDepth, SolidEntryPointCheckThresholdFuture)
    // 	snapshotDepth = SolidEntryPointCheckThresholdFuture
    // }
    // snapshotIntervalSynced = milestone.Index(config.NodeConfig.GetInt(config.CfgLocalSnapshotsIntervalSynced))
    // snapshotIntervalUnsynced = milestone.Index(config.NodeConfig.GetInt(config.CfgLocalSnapshotsIntervalUnsynced))
    //
    // pruningEnabled = config.NodeConfig.GetBool(config.CfgPruningEnabled)
    // pruningDelay = milestone.Index(config.NodeConfig.GetInt(config.CfgPruningDelay))
    // pruningDelayMin := snapshotDepth + SolidEntryPointCheckThresholdPast + AdditionalPruningThreshold + 1
    // if pruningDelay < pruningDelayMin {
    // 	log.Warnf("Parameter '%s' is too small (%d). Value was changed to %d", config.CfgPruningDelay, pruningDelay, pruningDelayMin)
    // 	pruningDelay = pruningDelayMin
    // }
    //
    // snapshotInfo := tangle.GetSnapshotInfo()
    // if snapshotInfo != nil {
    // 	coordinatorAddress := hornet.HashFromAddressTrytes(config.NodeConfig.GetString(config.CfgCoordinatorAddress))
    //
    // 	// Check coordinator address in database
    // 	if !bytes.Equal(snapshotInfo.CoordinatorAddress, coordinatorAddress) {
    // 		if !*overwriteCooAddress {
    // 			log.Panic(errors.Wrapf(ErrWrongCoordinatorAddressDatabase, "%v != %v", snapshotInfo.CoordinatorAddress.Trytes(), config.NodeConfig.GetString(config.CfgCoordinatorAddress)))
    // 		}
    //
    // 		// Overwrite old coordinator address
    // 		snapshotInfo.CoordinatorAddress = coordinatorAddress
    // 		tangle.SetSnapshotInfo(snapshotInfo)
    // 	}
    //
    // 	if !*forceGlobalSnapshot {
    // 		// If we don't enforce loading of a global snapshot,
    // 		// we can check the ledger state of current database and start the node.
    // 		tangle.GetLedgerStateForLSMI(nil)
    // 		return
    // 	}
    // }
    //
    // snapshotTypeToLoad := strings.ToLower(config.NodeConfig.GetString(config.CfgSnapshotLoadType))
    //
    // if *forceGlobalSnapshot && snapshotTypeToLoad != "global" {
    // 	log.Fatalf("global snapshot enforced but wrong snapshot type under config option '%s': %s", config.CfgSnapshotLoadType, config.NodeConfig.GetString(config.CfgSnapshotLoadType))
    // }

    match config.load_type() {
        config::LoadType::Global => {
            global::GlobalSnapshot::from_file(config.global().path(), MilestoneIndex(*config.global().index()));
        }
        config::LoadType::Local => {
            if !Path::new(config.local().path()).exists() {
                // TODO handle error
                local::download_local_snapshot(config.local());
                // 		err = LoadSnapshotFromFile(path)
            }
        }
    }

    bus.add_listener(on_last_solid_milestone_changed);
}
