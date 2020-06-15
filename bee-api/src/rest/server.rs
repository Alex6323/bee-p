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

use std::net::SocketAddr;

pub const SERVER_ADDRESS: &str = "127.0.0.1:3030";

pub async fn run(addr: SocketAddr) {

    let tx_by_hash = warp::post()
        .and(warp::path("v1"))
        .and(warp::path("transaction"))
        .and(warp::path::end())
        .and(json_body())
        .and_then(routes::transaction_by_hash);

    let routes = tx_by_hash;

    warp::serve(routes)
        .run(addr)
        .await;

}

fn json_body<T: DeserializeOwned + Send>() -> impl Filter<Extract = (T,), Error = Rejection> + Copy {
    warp::body::content_length_limit(1024 * 32).and(warp::body::json())
}