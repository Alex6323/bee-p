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
    event::MilestoneConfirmed,
    whiteflag::{
        traversal::{visit_bundles_dfs, Error as TraversalError},
        WhiteFlag,
    },
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_protocol::{Milestone, MilestoneIndex};

use futures::{channel::mpsc, stream::StreamExt};
use log::{error, info};

type Receiver = ShutdownStream<mpsc::UnboundedReceiver<LedgerConfirmationWorkerEvent>>;

enum Error {
    NonContiguousMilestone,
    InvalidConfirmationSet(TraversalError),
}

pub(crate) struct Confirmation {}

pub(crate) struct LedgerConfirmationWorkerEvent(pub(crate) Milestone);

pub(crate) struct LedgerConfirmationWorker {
    confirmed_index: MilestoneIndex,
    receiver: Receiver,
}

impl LedgerConfirmationWorker {
    pub fn new(confirmed_index: MilestoneIndex, receiver: Receiver) -> Self {
        Self {
            confirmed_index,
            receiver,
        }
    }

    fn confirm(&mut self, milestone: Milestone) -> Result<(), Error> {
        if milestone.index() != MilestoneIndex(self.confirmed_index.0 + 1) {
            error!(
                "Tried to confirm {} on top of {}.",
                milestone.index().0,
                self.confirmed_index.0
            );
            return Err(Error::NonContiguousMilestone);
        }

        info!("Confirming milestone {}.", milestone.index().0);

        match visit_bundles_dfs(*milestone.hash()) {
            Ok(_) => {
                self.confirmed_index = milestone.index();

                WhiteFlag::get().bus.dispatch(MilestoneConfirmed(milestone));

                Ok(())
            }
            Err(e) => {
                error!(
                    "Error occured while traversing to confirm {}: {:?}.",
                    milestone.index().0,
                    e
                );
                Err(Error::InvalidConfirmationSet(e))
            }
        }
    }

    pub async fn run(mut self) -> Result<(), WorkerError> {
        info!("Running.");

        while let Some(LedgerConfirmationWorkerEvent(milestone)) = self.receiver.next().await {
            if let Err(_) = self.confirm(milestone) {
                panic!("Error while confirming milestone, aborting.");
            }
        }

        info!("Stopped.");

        Ok(())
    }
}
