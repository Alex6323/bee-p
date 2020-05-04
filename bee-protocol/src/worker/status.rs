use crate::protocol::Protocol;

use bee_tangle::tangle;

use std::time::Duration;

use async_std::{future::ready, prelude::*};
use futures::channel::mpsc::Receiver;
use log::info;

pub(crate) struct StatusWorker {}

impl StatusWorker {
    pub(crate) fn new() -> Self {
        Self {}
    }

    fn status(&self) {
        let snapshot_milestone_index: u32 = *tangle().get_snapshot_milestone_index();
        let solid_milestone_index: u32 = *tangle().get_solid_milestone_index();
        let last_milestone_index: u32 = *tangle().get_last_milestone_index();

        // TODO Threshold
        // TODO use tangle synced method
        let mut status = if solid_milestone_index == last_milestone_index {
            String::from("Synchronized")
        } else {
            let progress = ((solid_milestone_index - snapshot_milestone_index) as f32 * 100.0
                / (last_milestone_index - snapshot_milestone_index) as f32) as u8;
            format!(
                "Synchronizing {}..{}..{} ({}%)",
                snapshot_milestone_index, solid_milestone_index, last_milestone_index, progress
            )
        };

        status = format!("{} Requested {}", status, Protocol::get().requested.len());

        info!("[StatusWorker ] {}.", status);
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
