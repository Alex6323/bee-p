use bee_common::{shutdown::ShutdownListener, worker::Error as WorkerError};

pub struct PeerDiscoveryWorker {
    shutdown: ShutdownListener,
}

impl PeerDiscoveryWorker {
    pub async fn run(mut self) -> Result<(), WorkerError> {
        loop {
            // select! {

            // }
        }
    }
}
