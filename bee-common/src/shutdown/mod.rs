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

// TODO remove
// use crate::{endpoint::whitelist, errors::Result};

mod error;

use async_std::task;
use futures::channel::oneshot;

pub use error::{Error, Result};

pub type ShutdownNotifier = oneshot::Sender<()>;
pub type ShutdownListener = oneshot::Receiver<()>;
pub type TaskHandle = task::JoinHandle<Result<()>>;

/// Handles the graceful shutdown of asynchronous tasks.
pub struct ShutdownHandler {
    notifiers: Vec<ShutdownNotifier>,
    tasks: Vec<TaskHandle>,
    actions: Vec<Box<dyn Fn()>>,
}

impl ShutdownHandler {
    /// Creates a new instance.
    pub fn new() -> Self {
        Self {
            notifiers: vec![],
            tasks: vec![],
            actions: vec![],
        }
    }

    /// Adds a shutdown notifier.
    pub fn add_notifier(&mut self, notifier: ShutdownNotifier) {
        self.notifiers.push(notifier);
    }

    /// Adds an asynchronous task.
    pub fn add_task(&mut self, task: TaskHandle) {
        self.tasks.push(task);
    }

    /// Adds an action that is applied during shutdown.
    pub fn add_action(&mut self, action: Box<dyn Fn()>) {
        self.actions.push(action);
    }

    /// Executes the shutdown.
    pub async fn execute(self) {
        for action in self.actions {
            action();
        }

        // TODO remove
        // whitelist::drop();

        let mut tasks = self.tasks;

        for notifier in self.notifiers {
            notifier.send(()).expect("error sending shutdown signal to task");
        }

        for task in &mut tasks {
            task.await.expect("error waiting for task to finish");
        }
    }
}
