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

use crate::worker::Error as WorkerError;

use futures::{channel::oneshot, future::FutureExt};
use log::error;
use thiserror::Error;

use std::future::Future;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Sending the shutdown signal to a worker failed.")]
    SendingShutdownSignalFailed,

    #[error("Waiting for worker to shut down failed.")]
    WaitingforWorkerShutdownFailed(#[from] WorkerError),
}

pub type ShutdownNotifier = oneshot::Sender<()>;
pub type ShutdownListener = oneshot::Receiver<()>;
pub type WorkerHandle = dyn Future<Output = Result<(), WorkerError>> + Unpin;

/// Handles the graceful shutdown of asynchronous workers.
pub struct Shutdown {
    notifiers: Vec<ShutdownNotifier>,
    workers: Vec<Box<WorkerHandle>>,
    actions: Vec<Box<dyn Fn()>>,
}

impl Shutdown {
    /// Creates a new instance.
    pub fn new() -> Self {
        Self {
            notifiers: Vec::new(),
            workers: Vec::new(),
            actions: Vec::new(),
        }
    }

    /// Adds a shutdown notifier.
    pub fn add_notifier(&mut self, notifier: ShutdownNotifier) {
        self.notifiers.push(notifier);
    }

    /// Adds an asynchronous worker.
    pub fn add_worker<T>(&mut self, worker: impl Future<Output = Result<T, WorkerError>> + Unpin + 'static) {
        self.workers.push(Box::new(worker.map(|x| x.map(|_| ()))));
    }

    /// Adds teardown logic that is executed during shutdown.
    pub fn add_action(&mut self, action: impl Fn() + 'static) {
        self.actions.push(Box::new(action));
    }

    /// Executes the shutdown.
    pub async fn execute(self) -> Result<(), Error> {
        for notifier in self.notifiers {
            // NOTE: in case of an error the `Err` variant simply contains our shutdown signal `()` that we tried to
            // send.
            notifier.send(()).map_err(|_| Error::SendingShutdownSignalFailed)?
        }

        for worker in self.workers {
            if let Err(e) = worker.await {
                error!("Awaiting worker failed: {:?}.", e);
            }
        }

        for action in self.actions {
            action();
        }

        Ok(())
    }
}
