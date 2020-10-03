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
use bee_transaction::Vertex;

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::{info, warn};

pub(crate) struct BundleValidatorWorkerEvent(pub(crate) Hash);

pub(crate) struct BundleValidatorWorker {
    pub(crate) tx: flume::Sender<BundleValidatorWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for BundleValidatorWorker {
    type Config = ();
    type Error = WorkerError;

    async fn start(node: &N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            while let Some(BundleValidatorWorkerEvent(hash)) = receiver.next().await {
                match load_bundle_builder(tangle(), &hash) {
                    Some(builder) => {
                        if let Ok(bundle) = builder.validate() {
                            tangle().update_metadata(&hash, |metadata| {
                                metadata.flags_mut().set_valid(true);
                            });
                            tangle().add_to_tip_pool(hash, *bundle.trunk(), *bundle.branch());
                        }
                    }
                    None => {
                        warn!("Failed to validate bundle: tail not found.");
                    }
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
