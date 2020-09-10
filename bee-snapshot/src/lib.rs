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

use bee_common::{shutdown::Shutdown, shutdown_stream::ShutdownStream};
use bee_common_ext::event::Bus;
use bee_protocol::{event::LastSolidMilestoneChanged, MilestoneIndex};

use async_std::task::spawn;
use futures::channel::{mpsc, oneshot};
use log::warn;

use std::{path::Path, sync::Arc};

pub fn init(config: &config::SnapshotConfig, bus: Arc<Bus<'static>>, shutdown: &mut Shutdown) {
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

    let (snapshot_worker_tx, snapshot_worker_rx) = mpsc::unbounded();
    let (snapshot_worker_shutdown_tx, snapshot_worker_shutdown_rx) = oneshot::channel();

    shutdown.add_worker_shutdown(
        snapshot_worker_shutdown_tx,
        spawn(
            worker::SnapshotWorker::new(
                config.clone(),
                ShutdownStream::new(snapshot_worker_shutdown_rx, snapshot_worker_rx),
            )
            .run(),
        ),
    );

    bus.add_listener(move |last_solid_milestone: &LastSolidMilestoneChanged| {
        if let Err(e) = snapshot_worker_tx.unbounded_send(worker::SnapshotWorkerEvent(last_solid_milestone.0.clone())) {
            warn!(
                "Failed to send milestone {} to snapshot worker: {:?}.",
                *last_solid_milestone.0.index(),
                e
            )
        }
    });
}
