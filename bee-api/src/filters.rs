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

use crate::handlers;
use bee_common_ext::node::ResHandle;
use bee_protocol::tangle::MsTangle;
use bee_storage::storage::Backend;
use serde::de::DeserializeOwned;
use std::str::FromStr;
use warp::{reject, Filter, Rejection};

pub fn all<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    get_info(tangle.clone()).or(get_milestones(tangle.clone()))
}

fn get_info<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("info"))
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::get_info)
}

fn get_milestones<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("milestones"))
        .and(custom_param::<u32>())
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::get_milestones)
}

/// Extract a denominator from a "div-by" header, or reject with DivideByZero.
fn custom_param<T: FromStr>() -> impl Filter<Extract = (T,), Error = Rejection> + Copy {
    warp::path::param().and_then(|value: String| async move {
        match value.parse::<T>() {
            Ok(x) => Ok(x),
            Err(_) => Err(reject::custom(BadRequest)),
        }
    })
}

#[derive(Debug)]
pub struct BadRequest;
impl reject::Reject for BadRequest {}

fn with_tangle<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = (ResHandle<MsTangle<B>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || tangle.clone())
}

fn json_body<T: DeserializeOwned + Send>() -> impl Filter<Extract = (T,), Error = Rejection> + Copy {
    warp::body::content_length_limit(1024 * 32).and(warp::body::json())
}
