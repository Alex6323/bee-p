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
                None => {
                    info!("[StatusWorker ] Status.");
                }
            }
        }

        info!("[StatusWorker ] Stopped.");
    }
}
