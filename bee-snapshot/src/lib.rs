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

use bee_protocol::MilestoneIndex;

use std::path::Path;

pub fn init(config: &config::SnapshotConfig) {
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
    //
    // var err = ErrNoSnapshotSpecified
    // switch snapshotTypeToLoad {
    // case "global":
    // 	if path := config.NodeConfig.GetString(config.CfgGlobalSnapshotPath); path != "" {
    // 		err = LoadGlobalSnapshot(path,
    // 			milestone.Index(config.NodeConfig.GetInt(config.CfgGlobalSnapshotIndex)))
    // 	}
    // case "local":
    // 	if path := config.NodeConfig.GetString(config.CfgLocalSnapshotsPath); path != "" {
    //
    // 		if _, fileErr := os.Stat(path); os.IsNotExist(fileErr) {
    // 			// create dir if it not exists
    // 			if err := os.MkdirAll(filepath.Dir(path), 0700); err != nil {
    // 				log.Fatalf("could not create snapshot dir '%s'", path)
    // 			}
    // 			if urls := config.NodeConfig.GetStringSlice(config.CfgLocalSnapshotsDownloadURLs); len(urls) > 0 {
    // 				log.Infof("Downloading snapshot from one of the provided sources %v", urls)
    // 				downloadErr := downloadSnapshotFile(path, urls)
    // 				if downloadErr != nil {
    // 					err = errors.Wrap(downloadErr, "Error downloading snapshot file")
    // 					break
    // 				}
    // 				log.Info("Snapshot download finished")
    // 			} else {
    // 				err = ErrNoSnapshotDownloadURL
    // 				break
    // 			}
    // 		}
    //
    // 		err = LoadSnapshotFromFile(path)
    // 	}
    // default:
    // 	log.Fatalf("invalid snapshot type under config option '%s': %s", config.CfgSnapshotLoadType, config.NodeConfig.GetString(config.CfgSnapshotLoadType))
    // }
    //
    // if err != nil {
    // 	tangle.MarkDatabaseCorrupted()
    // 	log.Panic(err.Error())
    // }

    match config.load_type() {
        config::LoadType::Global => {
            global::GlobalSnapshot::from_file(config.global().path(), MilestoneIndex(*config.global().index()));
        }
        config::LoadType::Local => {
            if !Path::new(config.local().path()).exists() {
                // TODO handle error
                local::download_local_snapshot(config.local());
            }
        }
    }
}
