use futures::{
    channel::{
        mpsc,
        oneshot,
    },
    future::FutureExt,
    select,
    stream::StreamExt,
};
use log::info;

pub struct LedgerWorkerEvent {}

#[derive(Default)]
pub struct LedgerWorker {}

impl LedgerWorker {
    pub fn new() -> Self {
        Self::default()
    }

    fn process(&self) {}

    pub async fn run(self, receiver: mpsc::Receiver<LedgerWorkerEvent>, shutdown: oneshot::Receiver<()>) {
        info!("[LedgerWorker ] Running.");

        let mut receiver_fused = receiver.fuse();
        let mut shutdown_fused = shutdown.fuse();

        loop {
            select! {
                event = receiver_fused.next() => {
                    if let Some(LedgerWorkerEvent{}) = event {
                        self.process();
                    }
                },
                _ = shutdown_fused => {
                    break;
                }
            }
        }

        info!("[LedgerWorker ] Stopped.");
    }
}
