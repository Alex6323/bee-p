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

use bee_common::{worker::Error as WorkerError};
use bee_common_ext::{node::Node, worker::Worker};

mod routes;
pub mod config;

use async_trait::async_trait;
use log::{info, error, warn};
use crate::config::{ApiConfig, ApiConfigBuilder};
use bee_common_ext::node::NodeBuilder;
use bee_common::worker::Error;
use bee_protocol::TangleWorker;
use std::any::TypeId;
use bee_protocol::tangle::MsTangle;
use futures::FutureExt;
use serde_json::Value as JsonValue;
use warp::Filter;


pub async fn init<N: Node> (
    config: ApiConfig,
    mut node_builder: N::Builder,
) -> N::Builder {
    node_builder.with_worker_cfg::<ApiWorker>(config)
}

pub struct ApiWorker;
#[async_trait]
impl<N: Node> Worker<N> for ApiWorker {
    type Config = ApiConfig;
    type Error = WorkerError;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<TangleWorker>()].leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {

        let tangle = node.resource::<MsTangle<N::Backend>>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let info = warp::get()
                .and(warp::path("api/v1"))
                .and(warp::path("info"))
                .and(warp::path::end())
                .and_then(move || {
                    let tangle = tangle.clone();
                    async move {
                        routes::info(tangle.clone()).await
                    }
                });

            let routes = info;

            let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(config.rest_socket_addr(), async {
                shutdown.await.ok();
            });

            info!("Stopped.");
        });

        Ok(Self)
    }

}

