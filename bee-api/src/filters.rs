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
use bee_message::MessageId;
use bee_protocol::{tangle::MsTangle, MilestoneIndex};
use bee_storage::storage::Backend;
use serde::de::DeserializeOwned;
use std::convert::TryFrom;
use warp::{reject, Filter, Rejection};

#[derive(Debug)]
pub struct BadRequest;
impl reject::Reject for BadRequest {}

#[derive(Debug)]
pub struct ServiceUnavailable;
impl reject::Reject for ServiceUnavailable {}

pub fn all<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    get_health(tangle.clone())
        .or(get_info(tangle.clone()).or(get_milestone_by_index(tangle.clone())))
        .or(get_tips(tangle.clone()))
        .or(get_message_by_id(tangle.clone()))
}

fn get_health<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("health"))
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::get_health)
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

fn get_tips<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("tips"))
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::get_tips)
}

fn get_message_by_id<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(message_id())
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::get_message_by_id)
}

fn get_milestone_by_index<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("milestones"))
        .and(milestone_index())
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::get_milestone_by_index)
}

fn message_id() -> impl Filter<Extract = (MessageId,), Error = Rejection> + Copy {
    warp::path::param().and_then(|value: String| async move {
        match MessageId::try_from(value.as_str()) {
            Ok(x) => Ok(x),
            Err(_) => Err(reject::custom(BadRequest)),
        }
    })
}

fn milestone_index() -> impl Filter<Extract = (MilestoneIndex,), Error = Rejection> + Copy {
    warp::path::param().and_then(|value: String| async move {
        match value.parse::<u32>() {
            Ok(x) => Ok(MilestoneIndex(x)),
            Err(_) => Err(reject::custom(BadRequest)),
        }
    })
}

fn with_tangle<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = (ResHandle<MsTangle<B>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || tangle.clone())
}

fn json_body<T: DeserializeOwned + Send>() -> impl Filter<Extract = (T,), Error = Rejection> + Copy {
    warp::body::content_length_limit(1024 * 32).and(warp::body::json())
}
