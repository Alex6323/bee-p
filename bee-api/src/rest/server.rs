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

use crate::rest::routes;

use futures::channel::oneshot;
use futures_util::FutureExt;
use serde::de::DeserializeOwned;

use warp::{Filter, Rejection};

use crate::{api::Api, config::ApiConfig};
use bee_common_ext::{shutdown::Shutdown, worker::Error as WorkerError};
use std::io::{Error, ErrorKind};

pub fn run(config: ApiConfig, shutdown: &mut Shutdown) {
    let node_info = warp::get()
        .and(warp::path("v1"))
        .and(warp::path("node-info"))
        .and(warp::path::end())
        .and_then(routes::RestApi::node_info);

    let tx_by_hash = warp::get()
        .and(warp::path("v1"))
        .and(warp::path("transaction"))
        .and(warp::path("by-hash"))
        .and(warp::path::param())
        .and(warp::path::end())
        .map(|param| serde_json::Value::String(param))
        .and_then(routes::RestApi::transaction_by_hash);

    let txs_by_hashes = warp::post()
        .and(warp::path("v1"))
        .and(warp::path("transaction"))
        .and(warp::path("by-hash"))
        .and(warp::path::end())
        .and(json_body())
        .and_then(routes::RestApi::transactions_by_hashes);

    let txs_by_bundle = warp::post()
        .and(warp::path("v1"))
        .and(warp::path("transaction"))
        .and(warp::path("by-bundle"))
        .and(warp::path::end())
        .and(json_body())
        .and_then(routes::RestApi::transactions_by_bundle);

    let routes = tx_by_hash
        .or(txs_by_hashes.or(node_info).or(txs_by_bundle))
        .with(warp::cors().allow_any_origin());

    let (server_sd_sender, server_sd_receiver) = oneshot::channel::<()>();

    let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(config.rest_socket_addr(), async {
        server_sd_receiver.await.ok();
    });

    let handle = tokio::spawn(server).map(|result| match result {
        Ok(_) => Ok(()),
        Err(_) => Err(WorkerError::AsynchronousOperationFailed(Error::new(
            ErrorKind::Other,
            "asynchronous operation failed",
        ))),
    });

    shutdown.add_worker_shutdown(server_sd_sender, handle)
}

fn json_body<T: DeserializeOwned + Send>() -> impl Filter<Extract = (T,), Error = Rejection> + Copy {
    warp::body::content_length_limit(1024 * 32).and(warp::body::json())
}
