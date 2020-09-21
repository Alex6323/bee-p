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

use crate::{
    config::SnapshotConfig,
    constants::{
        ADDITIONAL_PRUNING_THRESHOLD, SOLID_ENTRY_POINT_CHECK_THRESHOLD_FUTURE, SOLID_ENTRY_POINT_CHECK_THRESHOLD_PAST,
    },
    local::snapshot,
    pruning::prune_database,
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{node::Node, worker::Worker};
use bee_protocol::{tangle::tangle, Milestone, MilestoneIndex};

use async_trait::async_trait;
use futures::{
    channel::mpsc,
    stream::{Fuse, StreamExt},
};
use log::{error, info, warn};

pub(crate) struct SnapshotWorkerEvent(pub(crate) Milestone);

pub(crate) struct SnapshotWorker {
    config: SnapshotConfig,
    depth: u32,
    delay: u32,
}

#[async_trait]
impl<N: Node + 'static> Worker<N> for SnapshotWorker {
    type Event = SnapshotWorkerEvent;
    type Receiver = ShutdownStream<Fuse<mpsc::UnboundedReceiver<SnapshotWorkerEvent>>>;

    async fn run(mut self, mut receiver: Self::Receiver) -> Result<(), WorkerError> {
        info!("Running.");

        while let Some(SnapshotWorkerEvent(milestone)) = receiver.next().await {
            self.process(milestone);
        }

        info!("Stopped.");

        Ok(())
    }
}

impl SnapshotWorker {
    pub(crate) fn new(config: SnapshotConfig) -> Self {
        let depth = if config.local().depth() < SOLID_ENTRY_POINT_CHECK_THRESHOLD_FUTURE {
            warn!(
                "Configuration value for \"depth\" is too low ({}), value changed to {}.",
                config.local().depth(),
                SOLID_ENTRY_POINT_CHECK_THRESHOLD_FUTURE
            );
            SOLID_ENTRY_POINT_CHECK_THRESHOLD_FUTURE
        } else {
            config.local().depth()
        };
        let delay_min =
            config.local().depth() + SOLID_ENTRY_POINT_CHECK_THRESHOLD_PAST + ADDITIONAL_PRUNING_THRESHOLD + 1;
        let delay = if config.pruning().delay() < delay_min {
            warn!(
                "Configuration value for \"delay\" is too low ({}), value changed to {}.",
                config.pruning().delay(),
                delay_min
            );
            delay_min
        } else {
            config.pruning().delay()
        };

        Self { config, depth, delay }
    }

    fn should_snapshot(&self, index: MilestoneIndex) -> bool {
        let solid_index = *index;
        let snapshot_index = *tangle().get_snapshot_index();
        let pruning_index = *tangle().get_pruning_index();
        let snapshot_interval = if tangle().is_synced() {
            self.config.local().interval_synced()
        } else {
            self.config.local().interval_unsynced()
        };

        if (solid_index < self.depth + snapshot_interval)
            || (solid_index - self.depth) < pruning_index + 1 + SOLID_ENTRY_POINT_CHECK_THRESHOLD_PAST
        {
            // Not enough history to calculate solid entry points.
            return false;
        }

        return solid_index - (self.depth + snapshot_interval) >= snapshot_index;
    }

    fn should_prune(&self, mut index: MilestoneIndex) -> bool {
        if !self.config.pruning().enabled() {
            return false;
        }

        if *index <= self.delay {
            return false;
        }

        // Pruning happens after creating the snapshot so the metadata should provide the latest index.
        if *tangle().get_snapshot_index() < SOLID_ENTRY_POINT_CHECK_THRESHOLD_PAST + ADDITIONAL_PRUNING_THRESHOLD + 1 {
            return false;
        }

        let target_index_max = MilestoneIndex(
            *tangle().get_snapshot_index() - SOLID_ENTRY_POINT_CHECK_THRESHOLD_PAST - ADDITIONAL_PRUNING_THRESHOLD - 1,
        );

        if index > target_index_max {
            index = target_index_max;
        }

        if tangle().get_pruning_index() >= index {
            return false;
        }

        // We prune in "ADDITIONAL_PRUNING_THRESHOLD" steps to recalculate the solid_entry_points.
        if *tangle().get_entry_point_index() + ADDITIONAL_PRUNING_THRESHOLD + 1 > *index {
            return false;
        }

        true
    }

    fn process(&mut self, milestone: Milestone) {
        if self.should_snapshot(milestone.index()) {
            if let Err(e) = snapshot(self.config.local().path(), *milestone.index() - self.depth) {
                error!("Failed to create snapshot: {:?}.", e);
            }
        }
        if self.should_prune(milestone.index()) {
            if let Err(e) = prune_database(MilestoneIndex(*milestone.index() - self.delay)) {
                error!("Failed to prune database: {:?}.", e);
            }
        }
    }
}
