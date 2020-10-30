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

pub mod config;
mod filters;
mod handlers;
mod types;

use crate::{
    config::ApiConfig,
    types::{ErrorResponse, ErrorResponseBody},
};
use async_trait::async_trait;
use bee_common::worker::Error as WorkerError;
use bee_common_ext::{
    node::{Node, NodeBuilder},
    worker::Worker,
};
use bee_protocol::{tangle::MsTangle, TangleWorker};
use filters::{BadRequest, ServiceUnavailable};
use log::info;
use std::{any::TypeId, convert::Infallible};
use warp::{http::StatusCode, Filter, Rejection, Reply};

pub async fn init<N: Node>(config: ApiConfig, node_builder: N::Builder) -> N::Builder {
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

            let routes = filters::all(tangle).recover(handle_rejection);

            let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(config.binding_address(), async {
                shutdown.await.ok();
            });

            server.await;

            info!("Stopped.");
        });

        Ok(Self)
    }
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let http_code;
    let message_code;
    let message_text;

    if err.is_not_found() {
        http_code = StatusCode::NOT_FOUND;
        message_code = String::from("not_found");
        message_text = String::from("could not find data");
    } else if let Some(BadRequest) = err.find() {
        http_code = StatusCode::BAD_REQUEST;
        message_code = String::from("invalid_data");
        message_text = String::from("invalid data provided");
    } else if let Some(ServiceUnavailable) = err.find() {
        http_code = StatusCode::SERVICE_UNAVAILABLE;
        message_code = String::from("service_unavailable");
        message_text = String::from("service unavailable");
    } else {
        http_code = StatusCode::INTERNAL_SERVER_ERROR;
        message_code = String::from("internal_server_error");
        message_text = String::from("internal server error");
        eprintln!("unhandled rejection: {:?}", err);
    }

    Ok(warp::reply::with_status(
        warp::reply::json(&ErrorResponse::new(ErrorResponseBody {
            code: message_code,
            message: message_text,
        })),
        http_code,
    ))
}
