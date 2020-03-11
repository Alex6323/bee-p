use async_std::task;
use futures::channel::oneshot;

pub(crate) type ShutdownNotifier = oneshot::Sender<()>;
pub(crate) type ShutdownListener = oneshot::Receiver<()>;
pub(crate) type Task = task::JoinHandle<()>;

pub struct Shutdown {
    notifiers: Vec<ShutdownNotifier>,
    tasks: Vec<Task>,
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

    pub(crate) fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn num_tasks(&self) -> usize {
        self.tasks.len()
    }

    pub async fn execute(self) {
        let mut tasks = self.tasks;

        for notifier in self.notifiers {
            notifier.send(()).expect("error executing shutdown");
        }

        for task in &mut tasks {
            task.await;
        }
    }
}
