// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use crate::{tangle::tangle, worker::Worker};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_crypto::ternary::Hash;
use bee_tangle::helper::load_bundle_builder;

use async_trait::async_trait;
use futures::{
    channel::mpsc,
    stream::{Fuse, StreamExt},
};
use log::{info, warn};

use std::any::TypeId;

pub(crate) struct BundleValidatorWorkerEvent(pub(crate) Hash);

pub(crate) struct BundleValidatorWorker;

#[async_trait]
impl Worker for BundleValidatorWorker {
    const DEPS: &'static [TypeId] = &[];

    type Event = BundleValidatorWorkerEvent;
    type Receiver = ShutdownStream<Fuse<mpsc::UnboundedReceiver<Self::Event>>>;

    async fn run(mut self, mut receiver: Self::Receiver) -> Result<(), WorkerError> {
        info!("Running.");

        while let Some(BundleValidatorWorkerEvent(hash)) = receiver.next().await {
            self.validate(hash);
        }

        info!("Stopped.");

        Ok(())
    }
}

impl BundleValidatorWorker {
    pub(crate) fn new() -> Self {
        Self
    }

    fn validate(&self, tail_hash: Hash) {
        match load_bundle_builder(tangle(), &tail_hash) {
            Some(builder) => {
                if builder.validate().is_ok() {
                    tangle().update_metadata(&tail_hash, |metadata| {
                        metadata.flags.set_valid(true);
                    })
                }
            }
            None => {
                warn!("Faild to validate bundle: tail not found.");
            }
        }
    }
}
