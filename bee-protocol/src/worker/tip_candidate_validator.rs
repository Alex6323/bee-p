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

use async_trait::async_trait;
use futures::{channel::mpsc, stream::StreamExt};
use log::info;

use std::collections::HashMap;

pub(crate) struct BundleInfo {
    pub(crate) tail: Hash,
    pub(crate) trunk: Hash,
    pub(crate) branch: Hash,
}

pub(crate) enum TipCandidateValidatorWorkerEvent {
    BundleValidated(BundleInfo),
    TrsiPropagated(Hash),
}

pub(crate) struct TipCandidateValidatorWorker {
    pub(crate) tx: mpsc::UnboundedSender<TipCandidateValidatorWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for TipCandidateValidatorWorker {
    type Config = ();
    type Error = WorkerError;

    async fn start(node: &N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx);

            let mut not_ready_for_tip_pool = HashMap::new();

            while let Some(event) = receiver.next().await {
                match event {
                    TipCandidateValidatorWorkerEvent::BundleValidated(bundle_info) => {
                        if tangle().otrsi(&bundle_info.tail).is_some() && tangle().ytrsi(&bundle_info.tail).is_some() {
                            tangle().add_to_tip_pool(bundle_info.tail, bundle_info.trunk, bundle_info.branch);
                        } else {
                            not_ready_for_tip_pool.insert(bundle_info.tail, bundle_info);
                        }
                    }
                    TipCandidateValidatorWorkerEvent::TrsiPropagated(hash) => {
                        if let Some(bundle_info) = not_ready_for_tip_pool.remove(&hash) {
                            tangle().add_to_tip_pool(bundle_info.tail, bundle_info.trunk, bundle_info.branch);
                        }
                    }
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
