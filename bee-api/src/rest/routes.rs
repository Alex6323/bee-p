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

use serde_json::Value;

use crate::{format::utils, service::TransactionByHashParams};
use std::convert::TryFrom;

pub async fn node_info() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::json(&Value::from(ServiceImpl::node_info())))
}

pub async fn transaction_by_hash(json: Value) -> Result<impl warp::Reply, warp::Rejection> {
    match TransactionByHashParams::try_from(&json) {
        Ok(params) => Ok(warp::reply::json(&Value::from(ServiceImpl::transaction_by_hash(
            params,
        )))),
        Err(msg) => Ok(warp::reply::json(&utils::json_error(msg))),
    }
}
