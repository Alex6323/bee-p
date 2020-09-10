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

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_protocol::Milestone;

use futures::{
    channel::mpsc,
    stream::{Fuse, StreamExt},
};

use log::info;

type Receiver = ShutdownStream<Fuse<mpsc::UnboundedReceiver<SnapshotWorkerEvent>>>;

pub(crate) struct SnapshotWorkerEvent(pub(crate) Milestone);

pub(crate) struct SnapshotWorker {
    receiver: Receiver,
}

impl SnapshotWorker {
    pub(crate) fn new(receiver: Receiver) -> Self {
        Self { receiver }
    }

    fn process(&mut self, milestone: Milestone) {
        println!("{:?}", milestone.index());
    }

    pub(crate) async fn run(mut self) -> Result<(), WorkerError> {
        info!("Running.");

        while let Some(SnapshotWorkerEvent(milestone)) = self.receiver.next().await {
            self.process(milestone);
        }

        info!("Stopped.");

        Ok(())
    }
}
