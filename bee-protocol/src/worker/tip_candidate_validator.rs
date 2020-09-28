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

use crate::{event::TipCandidateFound, Protocol};
use bee_transaction::Vertex;
use std::{collections::HashMap, sync::Arc};

pub(crate) enum TipCandidateWorkerEvent {
    BundleValidated(BundleInfo),
    OtrsiYtrsiPropagated(Hash),
}

pub(crate) struct BundleInfo {
    pub(crate) tail: Hash,
    pub(crate) trunk: Hash,
    pub(crate) branch: Hash,
}

pub(crate) struct TipCandidateWorker {
    not_ready_for_tip_pool: HashMap<Hash, BundleInfo>,
}

#[async_trait]
impl<N: Node> Worker<N> for TipCandidateWorker {
    type Config = ();
    type Error = WorkerError;
    type Event = TipCandidateWorkerEvent;
    type Receiver = ShutdownStream<Fuse<mpsc::UnboundedReceiver<Self::Event>>>;

    async fn start(
        mut self,
        mut receiver: Self::Receiver,
        _node: Arc<N>,
        _config: Self::Config,
    ) -> Result<(), Self::Error> {
        info!("Running.");

        while let Some(event) = receiver.next().await {
            self.process(event);
        }

        info!("Stopped.");

        Ok(())
    }
}

impl TipCandidateWorker {
    pub(crate) fn new() -> Self {
        Self {
            not_ready_for_tip_pool: HashMap::new(),
        }
    }

    fn process(&mut self, event: TipCandidateWorkerEvent) {
        match event {
            TipCandidateWorkerEvent::BundleValidated(bundle_info) => {
                if tangle().otrsi(&bundle_info.tail).is_some() && tangle().ytrsi(&bundle_info.tail).is_some() {
                    Protocol::get().bus.dispatch(TipCandidateFound {
                        tail: bundle_info.tail,
                        trunk: bundle_info.trunk,
                        branch: bundle_info.branch,
                    });
                } else {
                    self.not_ready_for_tip_pool.insert(bundle_info.tail, bundle_info);
                }
            }
            TipCandidateWorkerEvent::OtrsiYtrsiPropagated(hash) => {
                if let Some(bundle_info) = self.not_ready_for_tip_pool.remove(&hash) {
                    Protocol::get().bus.dispatch(TipCandidateFound {
                        tail: bundle_info.tail,
                        trunk: bundle_info.trunk,
                        branch: bundle_info.branch,
                    });
                }
            }
        }
    }
}
