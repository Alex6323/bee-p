use bee_tangle::tangle;

use std::time::Duration;

use async_std::{
    future::ready,
    prelude::*,
};
use futures::channel::mpsc::Receiver;
use log::info;

pub(crate) struct StatusWorker {}

impl StatusWorker {
    pub(crate) fn new() -> Self {
        Self {}
    }

    fn status(&self) {
        let first_solid_milestone_index: u32 = *tangle().get_first_solid_milestone_index();
        let last_solid_milestone_index: u32 = *tangle().get_last_solid_milestone_index();
        let last_milestone_index: u32 = *tangle().get_last_milestone_index();

        // TODO Threshold
        if last_solid_milestone_index == last_milestone_index {
            info!("[StatusWorker ] Synchronized.");
        } else {
            // TODO %
            info!(
                "[StatusWorker ] Synchronizing {}..{}..{}.",
                first_solid_milestone_index, last_solid_milestone_index, last_milestone_index
            );
        }
    }

    pub(crate) async fn run(self, mut shutdown: Receiver<()>) {
        info!("[StatusWorker ] Running.");

        loop {
            match ready(None)
                .delay(Duration::from_millis(5000))
                .race(shutdown.next())
                .await
            {
                Some(_) => {
                    break;
                }
                None => self.status(),
            }
        }

        info!("[StatusWorker ] Stopped.");
    }
}
