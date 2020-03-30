use crate::endpoint::whitelist;
use crate::errors::Result;

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

        whitelist::free();

        let mut tasks = self.tasks;

        for notifier in self.notifiers {
            notifier.send(()).expect("error sending shutdown signal to task");
        }

        for task in &mut tasks {
            task.await.expect("error waiting for task to finish");
        }
    }
}
