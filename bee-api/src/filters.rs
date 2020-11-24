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
use serde::de::DeserializeOwned;
use warp::{reject, Filter, Rejection};

use bee_protocol::{tangle::MsTangle, MessageSubmitterWorkerEvent};

use crate::storage::Backend;
use std::collections::HashMap;

#[derive(Debug)]
pub struct BadRequest;
impl reject::Reject for BadRequest {}

#[derive(Debug)]
pub struct ServiceUnavailable;
impl reject::Reject for ServiceUnavailable {}

pub fn all<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
    storage: ResHandle<B>,
    message_submitter: flume::Sender<MessageSubmitterWorkerEvent>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    get_health(tangle.clone())
        .or(get_info(tangle.clone()).or(get_milestone_by_milestone_index(tangle.clone())))
        .or(get_tips(tangle.clone()))
        .or(post_raw_message(message_submitter))
        .or(get_message_by_index(storage.clone()))
        .or(get_message_by_message_id(tangle.clone()))
        .or(get_message_metadata(tangle.clone()))
        .or(get_raw_message(tangle.clone()))
        .or(get_children_by_message_id(tangle.clone()))
        .or(get_output_by_output_id(storage.clone()))
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

fn post_raw_message(
    message_submitter: flume::Sender<MessageSubmitterWorkerEvent>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(warp::path::end())
        .and(warp::body::bytes())
        .and(with_message_submitter(message_submitter))
        .and_then(handlers::post_message_raw)
}

fn get_message_by_index<B: Backend>(
    storage: ResHandle<B>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(warp::path::end())
        .and(warp::query().and_then(|query: HashMap<String, String>| async move {
            match query.get("index") {
                Some(i) => Ok(i.to_string()),
                None => Err(reject::custom(BadRequest)),
            }
        }))
        .and(with_storage(storage))
        .and_then(handlers::get_message_by_index)
}

fn get_message_by_message_id<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(custom_path_param::message_id())
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::get_message_by_message_id)
}

fn get_message_metadata<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(custom_path_param::message_id())
        .and(warp::path("metadata"))
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::get_message_metadata)
}

fn get_raw_message<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(custom_path_param::message_id())
        .and(warp::path("raw"))
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::get_raw_message)
}

fn get_children_by_message_id<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(custom_path_param::message_id())
        .and(warp::path("children"))
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::get_children_by_message_id)
}

fn get_milestone_by_milestone_index<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("milestones"))
        .and(custom_path_param::milestone_index())
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::get_milestone_by_milestone_index)
}

fn get_output_by_output_id<B: Backend>(
    storage: ResHandle<B>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("outputs"))
        .and(custom_path_param::output_id())
        .and(warp::path::end())
        .and(with_storage(storage))
        .and_then(handlers::get_output_by_output_id)
}

mod custom_path_param {

    use super::*;
    use bee_message::{payload::transaction::OutputId, MessageId};
    use bee_protocol::MilestoneIndex;

    pub(super) fn output_id() -> impl Filter<Extract = (OutputId,), Error = Rejection> + Copy {
        warp::path::param().and_then(|value: String| async move {
            match value.parse::<OutputId>() {
                Ok(id) => Ok(id),
                Err(_) => Err(reject::custom(BadRequest)),
            }
        })
    }

    pub(super) fn message_id() -> impl Filter<Extract = (MessageId,), Error = Rejection> + Copy {
        warp::path::param().and_then(|value: String| async move {
            match value.parse::<MessageId>() {
                Ok(msg) => Ok(msg),
                Err(_) => Err(reject::custom(BadRequest)),
            }
        })
    }

    pub(super) fn milestone_index() -> impl Filter<Extract = (MilestoneIndex,), Error = Rejection> + Copy {
        warp::path::param().and_then(|value: String| async move {
            match value.parse::<u32>() {
                Ok(i) => Ok(MilestoneIndex(i)),
                Err(_) => Err(reject::custom(BadRequest)),
            }
        })
    }
}

fn with_tangle<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
) -> impl Filter<Extract = (ResHandle<MsTangle<B>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || tangle.clone())
}

fn with_storage<B: Backend>(
    storage: ResHandle<B>,
) -> impl Filter<Extract = (ResHandle<B>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || storage.clone())
}

fn with_message_submitter(
    message_submitter: flume::Sender<MessageSubmitterWorkerEvent>,
) -> impl Filter<Extract = (flume::Sender<MessageSubmitterWorkerEvent>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || message_submitter.clone())
}


fn json_body<T: DeserializeOwned + Send>() -> impl Filter<Extract = (T,), Error = Rejection> + Copy {
    warp::body::content_length_limit(1024 * 32).and(warp::body::json())
}

fn raw_bytes() -> impl Filter<Extract = (warp::hyper::body::Bytes,), Error = Rejection> + Copy {
    warp::body::content_length_limit(1024 * 32).and(warp::body::bytes())
}
