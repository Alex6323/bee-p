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

use crate::tangle::tangle;

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_crypto::ternary::Hash;
use bee_tangle::helper::load_bundle_builder;

use futures::{
    channel::mpsc,
    stream::{Fuse, StreamExt},
};
use log::{error, info, warn};
use crate::Protocol;
use crate::event::{BundleValidated, TransactionSolidified};
use bee_transaction::Vertex;

type Receiver = ShutdownStream<Fuse<mpsc::UnboundedReceiver<BundleValidatorWorkerEvent>>>;

pub(crate) struct BundleValidatorWorkerEvent(pub(crate) Hash);

pub(crate) struct BundleValidatorWorker {
    receiver: Receiver,
}

impl BundleValidatorWorker {
    pub(crate) fn new(receiver: Receiver) -> Self {
        Self { receiver }
    }

    fn validate(&self, tail_hash: Hash) {
        match load_bundle_builder(tangle(), &tail_hash) {
            Some(builder) => {
                match builder.validate() {
                    Ok(validated) => {
                        tangle().update_metadata(&tail_hash, |metadata| {
                            metadata.flags.set_valid(true);
                        });
                        Protocol::get().bus.dispatch(BundleValidated {
                            tail_hash,
                            trunk: *validated.trunk(),
                            branch: *validated.branch()
                        });
                    }
                    Err(_) => { ; }
                }
            }
            None => {
                warn!("Failed to validate bundle: tail not found.");
            }
        }
    }

    pub(crate) async fn run(mut self) -> Result<(), WorkerError> {
        info!("Running.");

        Protocol::get().bus.add_listener(on_transaction_solidified);

        while let Some(BundleValidatorWorkerEvent(hash)) = self.receiver.next().await {
            self.validate(hash);
        }

        info!("Stopped.");

        Ok(())
    }

}

pub fn on_transaction_solidified(event: &TransactionSolidified) {
    if let Err(e) = Protocol::get()
        .bundle_validator_worker
        .unbounded_send(BundleValidatorWorkerEvent(event.0))
    {
        error!("Failed to send hash to bundle validator: {:?}.", e);
    }
}

