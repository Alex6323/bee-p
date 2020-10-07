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

use async_trait::async_trait;
use futures::StreamExt;
use log::info;
use tokio::time::interval;

use std::time::Duration;

#[derive(Default)]
pub(crate) struct TipPoolCleanerWorker {}

#[async_trait]
impl<N: Node> Worker<N> for TipPoolCleanerWorker {
    type Config = ();
    type Error = WorkerError;

    async fn start(node: &N, _config: Self::Config) -> Result<Self, Self::Error> {
        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, interval(Duration::from_secs(1)));

            while receiver.next().await.is_some() {
                tangle().reduce_tips()
            }

            info!("Stopped.");
        });

        Ok(Self::default())
    }
}
