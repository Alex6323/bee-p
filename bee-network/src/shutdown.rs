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

use crate::{endpoint::whitelist, errors::Result};

use async_std::task;
use futures::channel::oneshot;

pub(crate) type ShutdownNotifier = oneshot::Sender<()>;
pub(crate) type ShutdownListener = oneshot::Receiver<()>;
pub(crate) type WorkerTask = task::JoinHandle<Result<()>>;

/// Handles the graceful shutdown of the network layer.
pub struct Shutdown {
    notifiers: Vec<ShutdownNotifier>,
    tasks: Vec<WorkerTask>,
}

impl Shutdown {
    pub(crate) fn new() -> Self {
        Self {
            notifiers: vec![],
            tasks: vec![],
        }
    }

    pub(crate) fn add_notifier(&mut self, notifier: ShutdownNotifier) {
        self.notifiers.push(notifier);
    }

    pub(crate) fn add_task(&mut self, task: WorkerTask) {
        self.tasks.push(task);
    }

    /// Executes the shutdown.
    pub async fn execute(self) {
        whitelist::drop();

        let mut tasks = self.tasks;

        for notifier in self.notifiers {
            notifier.send(()).expect("error sending shutdown signal to task");
        }

        for task in &mut tasks {
            task.await.expect("error waiting for task to finish");
        }
    }
}
