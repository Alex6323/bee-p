use futures::{
    channel::oneshot,
    future::FutureExt,
    select,
};
use log::info;

pub(crate) struct StatusWorker {}

impl StatusWorker {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) async fn run(self, shutdown: oneshot::Receiver<()>) {
        info!("[StatusWorker ] Running.");

        let mut shutdown_fused = shutdown.fuse();

        loop {
            select! {
                _ = shutdown_fused => {
                    break;
                }
            }
        }

        info!("[StatusWorker ] Stopped.");
    }
}
