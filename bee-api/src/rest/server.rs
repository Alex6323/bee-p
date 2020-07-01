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

use serde::de::DeserializeOwned;
use warp::{Filter, Rejection};

use crate::config::ApiConfig;

pub async fn run(config: ApiConfig) {
    let node_info = warp::get()
        .and(warp::path("v1"))
        .and(warp::path("node-info"))
        .and(warp::path::end())
        .and_then(routes::node_info);

    let tx_by_hash_post = warp::post()
        .and(warp::path("v1"))
        .and(warp::path("transaction"))
        .and(warp::path("by-hash"))
        .and(warp::path::end())
        .and(json_body())
        .and_then(routes::transactions_by_hashes);

    let tx_by_hash_get = warp::get()
        .and(warp::path("v1"))
        .and(warp::path("transaction"))
        .and(warp::path("by-hash"))
        .and(warp::path::param())
        .and(warp::path::end())
        .and_then(routes::transaction_by_hash);

    let txs_by_bundle = warp::get()
        .and(warp::path("v1"))
        .and(warp::path("transaction"))
        .and(warp::path("by-bundle"))
        .and(warp::path::param())
        .and(warp::path::end())
        .and_then(routes::transactions_by_bundle);

    let routes = tx_by_hash_get
        .or(tx_by_hash_post.or(node_info))
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(config.rest_socket_addr()).await;
}

fn json_body<T: DeserializeOwned + Send>() -> impl Filter<Extract = (T,), Error = Rejection> + Copy {
    warp::body::content_length_limit(1024 * 32).and(warp::body::json())
}
