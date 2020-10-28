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
pub mod dto;
mod routes;

use std::sync::Arc;
use warp::http::StatusCode;
use crate::config::ApiConfig;
use async_trait::async_trait;
use bee_common::worker::Error as WorkerError;
use bee_common_ext::{
    node::{Node, NodeBuilder},
    worker::Worker,
};
use bee_protocol::{tangle::MsTangle, TangleWorker, MilestoneIndex};
use futures::FutureExt;
use log::{error, info, warn};
use serde_json::Value as JsonValue;
use std::any::TypeId;
use warp::{Filter, Rejection, reject, Reply};
use serde::de::DeserializeOwned;
use std::{
    convert::Infallible,
    time::{SystemTime, UNIX_EPOCH},
};
use crate::dto::{ErrorResponseBody, ErrorResponse};
use warp::filters::body::{BodyDeserializeError};


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

            let tangle_clone = tangle.clone();
            let get_info = warp::get()
                .and(warp::path("api"))
                .and(warp::path("v1"))
                .and(warp::path("info"))
                .and(warp::path::end())
                .and_then(move || {
                    let tangle = tangle_clone.clone();
                    async move { routes::get_info(tangle).await }
                });

            let tangle_clone = tangle.clone();
            let get_milestones = warp::get()
                .and(warp::path("api"))
                .and(warp::path("v1"))
                .and(warp::path("milestones"))
                .and(warp::path::param())
                .and(warp::path::end())
                .and_then(move | milestone_index: u32 | {
                    let tangle = tangle_clone.clone();
                    async move { routes::get_milestones(tangle, milestone_index).await }
                });

            let routes = get_info.or(get_milestones).with(warp::cors().allow_any_origin()).recover(handle_rejection);

            let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(config.binding_address(), async {
                shutdown.await.ok();
            });

            server.await;

            info!("Stopped.");
        });

        Ok(Self)
    }
}

fn json_body<T: DeserializeOwned + Send>() -> impl Filter<Extract = (T,), Error = Rejection> + Copy {
    warp::body::content_length_limit(1024 * 32).and(warp::body::json())
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {

    let http_code;
    let message_code;
    let message_text;

    if err.is_not_found() {
        http_code = StatusCode::NOT_FOUND;
        message_code = String::from("not_found");
        message_text = String::from("could not find data");
    } else {
        eprintln!("unhandled rejection: {:?}", err);
        http_code = StatusCode::INTERNAL_SERVER_ERROR;
        message_code = String::from("internal_server_error");
        message_text = String::from("internal server error");
    }

    Ok(warp::reply::with_status(warp::reply::json(&ErrorResponse::new(ErrorResponseBody {
        code: message_code,
        message: message_text,
    })), http_code))

}