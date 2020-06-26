use crate::events::EventSender;

use bee_common::{shutdown::ShutdownListener, worker::Error as WorkerError};

use futures::{select, FutureExt};

pub struct PeerDiscoveryWorker {
    shutdown_receiver: ShutdownListener,
    event_sender: EventSender,
}

impl PeerDiscoveryWorker {
    pub fn new(event_sender: EventSender, shutdown_receiver: ShutdownListener) -> Self {
        Self {
            event_sender,
            shutdown_receiver,
        }
    }

    pub async fn run(mut self) -> Result<(), WorkerError> {
        let mut shutdown_receiver = self.shutdown_receiver.fuse();
        let mut event_sender = self.event_sender;

        loop {
            select! {
                shutdown = shutdown_receiver => {
                    break;
                }
            }
        }

        Ok(())
    }
}
