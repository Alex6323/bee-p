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

use crate::service::{Service, ServiceImpl, TransactionsByBundleParams, VisitChildrenFollowTrunkParams};

use serde_json::Value as JsonValue;

use async_trait::async_trait;

use crate::{
    format::json_utils::{json_error_obj, json_success_obj},
    service::{TransactionByHashParams, TransactionsByHashesParams},
    web_service::WebService,
};
use std::convert::TryFrom;

pub struct Rest;
#[async_trait]
impl WebService for Rest {
    type Input = JsonValue;
    type Output = Result<warp::reply::Json, warp::Rejection>;

    async fn node_info() -> Self::Output {
        Ok(warp::reply::json(&JsonValue::from(ServiceImpl::node_info())))
    }

    async fn transactions_by_bundle(input: Self::Input) -> Self::Output {
        match TransactionsByBundleParams::try_from(&input) {
            Ok(params) => match ServiceImpl::transactions_by_bundle(params) {
                Ok(res) => Ok(warp::reply::json(&json_success_obj(res.into()))),
                Err(e) => Ok(warp::reply::json(&json_error_obj(&e.msg))),
            },
            Err(msg) => Ok(warp::reply::json(&json_error_obj(msg))),
        }
    }

    async fn transaction_by_hash(input: Self::Input) -> Self::Output {
        match TransactionByHashParams::try_from(&input) {
            Ok(params) => Ok(warp::reply::json(&json_success_obj(
                ServiceImpl::transaction_by_hash(params).into(),
            ))),
            Err(msg) => Ok(warp::reply::json(&json_error_obj(msg))),
        }
    }

    async fn transactions_by_hashes(input: Self::Input) -> Self::Output {
        match TransactionsByHashesParams::try_from(&input) {
            Ok(params) => Ok(warp::reply::json(&json_success_obj(
                ServiceImpl::transactions_by_hashes(params).into(),
            ))),
            Err(msg) => Ok(warp::reply::json(&json_error_obj(msg))),
        }
    }

    async fn visit_children_follow_trunk(input: Self::Input) -> Self::Output {
        match VisitChildrenFollowTrunkParams::try_from(&input) {
            Ok(params) => Ok(warp::reply::json(&json_success_obj(
                ServiceImpl::visit_children_follow_trunk(params).into(),
            ))),
            Err(msg) => Ok(warp::reply::json(&json_error_obj(msg))),
        }
    }
}
