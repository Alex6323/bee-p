//! Autopeering crate

use bee_common::{shutdown::Shutdown, worker::Error as WorkerError};

pub mod config;
pub mod events;
pub mod peers;
pub mod salt;

mod discover;

use async_std::task::spawn;
use config::AutopeeringConfig;
use discover::worker::PeerDiscoveryWorker;
use events::EventReceiver;
use futures::channel::oneshot;
use log::info;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("An asynchronous operation failed")]
    AsynchronousOperationFailed(#[from] WorkerError),
}

pub fn init(config: AutopeeringConfig, shutdown: &mut Shutdown) -> Result<EventReceiver, Error> {
    let (shutdown_sender, shutdown_receiver) = oneshot::channel();
    let (event_sender, event_receiver) = events::event_channel();

    shutdown.add_worker(spawn(PeerDiscoveryWorker::new(event_sender, shutdown_receiver).run()));

    shutdown.add_notifier(shutdown_sender);

    shutdown.add_action(Box::new(|| info!("Shutting down autopeering...")));

    Ok(event_receiver)
}
