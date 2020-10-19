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

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{node::Node, worker::Worker};

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::info;
use tokio::time::interval;

use std::time::Duration;

const _HEARTBEAT_SEND_INTERVAL_SEC: u64 = 30;
const _HEARTBEAT_RECEIVE_INTERVAL_SEC: u64 = 100;
const CHECK_HEARTBEATS_INTERVAL_SEC: u64 = 5;

#[derive(Default)]
pub(crate) struct HeartbeaterWorker {}

#[async_trait]
impl<N: Node> Worker<N> for HeartbeaterWorker {
    type Config = ();
    type Error = WorkerError;

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver =
                ShutdownStream::new(shutdown, interval(Duration::from_secs(CHECK_HEARTBEATS_INTERVAL_SEC)));

            while receiver.next().await.is_some() {
                // TODO impl
            }

            info!("Stopped.");
        });

        Ok(Self::default())
    }
}
