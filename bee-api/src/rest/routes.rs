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

use crate::service::{Service, ServiceImpl};

use serde_json::Value as JsonValue;

use crate::{
    format::json_utils::{json_error_obj, json_success_obj},
    service::{TransactionByHashParams, TransactionsByHashesParams},
};
use std::convert::TryFrom;

pub async fn node_info() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::json(&JsonValue::from(ServiceImpl::node_info())))
}

pub async fn transactions_by_hashes(json: JsonValue) -> Result<impl warp::Reply, warp::Rejection> {
    match TransactionsByHashesParams::try_from(&json) {
        Ok(params) => Ok(warp::reply::json(&json_success_obj(
            ServiceImpl::transactions_by_hashes(params).into(),
        ))),
        Err(msg) => Ok(warp::reply::json(&json_error_obj(
            msg,
            warp::http::StatusCode::BAD_REQUEST.as_u16(),
        ))),
    }
}

pub async fn transaction_by_hash(value: String) -> Result<impl warp::Reply, warp::Rejection> {
    match TransactionByHashParams::try_from(value.as_str()) {
        Ok(params) => Ok(warp::reply::json(&json_success_obj(
            ServiceImpl::transaction_by_hash(params).into(),
        ))),
        Err(msg) => Ok(warp::reply::json(&json_error_obj(
            msg,
            warp::http::StatusCode::BAD_REQUEST.as_u16(),
        ))),
    }
}
