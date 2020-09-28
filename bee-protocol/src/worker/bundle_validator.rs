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
use bee_common_ext::{node::Node, worker::Worker};
use bee_crypto::ternary::Hash;
use bee_tangle::helper::load_bundle_builder;

use async_trait::async_trait;
use futures::{
    channel::mpsc,
    stream::{Fuse, StreamExt},
};
use log::{info, warn};

use crate::{
    worker::tip_candidate_validator::{BundleInfo, TipCandidateWorkerEvent},
    Protocol,
};
use bee_transaction::Vertex;
use std::sync::Arc;

pub(crate) struct BundleValidatorWorkerEvent(pub(crate) Hash);

pub(crate) struct BundleValidatorWorker {
    tip_candidate_validator: mpsc::UnboundedSender<TipCandidateWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for BundleValidatorWorker {
    type Config = ();
    type Error = WorkerError;
    type Event = BundleValidatorWorkerEvent;
    type Receiver = ShutdownStream<Fuse<mpsc::UnboundedReceiver<Self::Event>>>;

    async fn start(
        mut self,
        mut receiver: Self::Receiver,
        _node: Arc<N>,
        _config: Self::Config,
    ) -> Result<(), Self::Error> {
        info!("Running.");

        while let Some(BundleValidatorWorkerEvent(hash)) = receiver.next().await {
            self.validate(hash);
        }

        info!("Stopped.");

        Ok(())
    }
}

impl BundleValidatorWorker {
    pub(crate) fn new(tip_candidate_validator: mpsc::UnboundedSender<TipCandidateWorkerEvent>) -> Self {
        Self {
            tip_candidate_validator,
        }
    }

    fn validate(&self, tail: Hash) {
        match load_bundle_builder(tangle(), &tail) {
            Some(builder) => {
                if let Ok(validated_bundle) = builder.validate() {
                    tangle().update_metadata(&tail, |metadata| {
                        metadata.flags_mut().set_valid(true);
                    });
                    if let Err(e) =
                        self.tip_candidate_validator
                            .unbounded_send(TipCandidateWorkerEvent::BundleValidated(BundleInfo {
                                tail,
                                trunk: *validated_bundle.trunk(),
                                branch: *validated_bundle.branch(),
                            }))
                    {
                        warn!("Failed to send hash to tip candidate validator: {:?}.", e);
                    }
                }
            }
            None => {
                warn!("Failed to validate bundle: tail not found.");
            }
        }
    }
}
