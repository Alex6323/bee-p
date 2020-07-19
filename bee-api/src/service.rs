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

use crate::format::items::transaction_ref::TransactionRefItem;
use bee_crypto::ternary::Hash;
use bee_protocol::{tangle::tangle, MilestoneIndex};
use bee_tangle::{traversal, TransactionRef};
use serde_json::Value as JsonValue;
use std::{collections::HashMap, fmt};

pub trait Service {
    fn node_info() -> NodeInfoResponse;
    fn transactions_by_bundle(params: TransactionsByBundleParams)
        -> Result<TransactionsByBundleResponse, ServiceError>;
    fn transaction_by_hash(params: TransactionByHashParams) -> TransactionByHashResponse;
    fn transactions_by_hashes(params: TransactionsByHashesParams) -> TransactionsByHashesResponse;
    fn visit_children_follow_trunk(params: VisitChildrenFollowTrunkParams) -> VisitChildrenFollowTrunkResponse;
}

pub struct ServiceError {
    pub msg: String,
}

impl ServiceError {
    fn new(msg: String) -> Self {
        Self { msg }
    }
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

pub struct NodeInfoResponse {
    pub is_synced: bool,
    pub last_milestone_index: MilestoneIndex,
    pub last_milestone_hash: Option<Hash>,
    pub last_solid_milestone_index: MilestoneIndex,
    pub last_solid_milestone_hash: Option<Hash>,
}

pub struct TransactionByHashParams {
    pub hash: Hash,
}

pub struct TransactionByHashResponse {
    pub tx_ref: Option<TransactionRef>,
}

pub struct TransactionsByHashesParams {
    pub hashes: Vec<Hash>,
}

pub struct TransactionsByHashesResponse {
    pub tx_refs: HashMap<Hash, Option<TransactionRef>>,
}

pub struct TransactionsByBundleParams {
    pub entry: Hash,
    pub bundle: Hash,
}

pub struct TransactionsByBundleResponse {
    pub tx_refs: HashMap<Hash, TransactionRef>,
}

pub struct VisitChildrenFollowTrunkParams {
    pub entry: Hash,
    pub traverse_cond: JsonValue,
    pub collect_cond: JsonValue,
}

pub struct VisitChildrenFollowTrunkResponse {
    pub tx_refs: HashMap<Hash, TransactionRef>,
}

pub struct ServiceImpl;
impl Service for ServiceImpl {
    fn node_info() -> NodeInfoResponse {
        NodeInfoResponse {
            is_synced: tangle().is_synced(),
            last_milestone_index: tangle().get_last_milestone_index(),
            last_milestone_hash: tangle().get_milestone_hash(tangle().get_last_milestone_index()),
            last_solid_milestone_index: tangle().get_last_solid_milestone_index(),
            last_solid_milestone_hash: tangle().get_milestone_hash(tangle().get_last_solid_milestone_index()),
        }
    }

    fn transactions_by_bundle(
        params: TransactionsByBundleParams,
    ) -> Result<TransactionsByBundleResponse, ServiceError> {
        let mut ret = HashMap::new();
        if params.entry == params.bundle {
            return Err(ServiceError::new(String::from("entry hash is equal to bundle hash")));
        }
        traversal::visit_children_depth_first(
            tangle(),
            params.entry,
            |tx_ref, _| tx_ref.bundle() == &params.bundle,
            |tx_hash, tx_ref, _| {
                ret.insert(tx_hash.clone(), tx_ref.clone());
            },
            |_| (),
        );
        Ok(TransactionsByBundleResponse { tx_refs: ret })
    }

    fn transaction_by_hash(params: TransactionByHashParams) -> TransactionByHashResponse {
        TransactionByHashResponse {
            tx_ref: tangle().get(&params.hash),
        }
    }

    fn transactions_by_hashes(params: TransactionsByHashesParams) -> TransactionsByHashesResponse {
        let mut ret = HashMap::new();
        for hash in params.hashes {
            ret.insert(
                hash,
                ServiceImpl::transaction_by_hash(TransactionByHashParams { hash }).tx_ref,
            );
        }
        TransactionsByHashesResponse { tx_refs: ret }
    }

    fn visit_children_follow_trunk(params: VisitChildrenFollowTrunkParams) -> VisitChildrenFollowTrunkResponse {
        let mut ret = HashMap::new();
        traversal::visit_children_follow_trunk(
            tangle(),
            params.entry,
            |tx_ref, _| match_cond(&params.traverse_cond, tx_ref),
            |tx_hash, tx_ref, _| {
                if match_cond(&params.collect_cond, tx_ref) {
                    ret.insert(tx_hash.clone(), tx_ref.clone());
                }
            },
        );
        VisitChildrenFollowTrunkResponse { tx_refs: ret }
    }
}

pub fn match_cond(cond: &JsonValue, tx_ref: &TransactionRef) -> bool {
    match jsonlogic_rs::apply(cond, &JsonValue::from(&TransactionRefItem(tx_ref.clone()))) {
        Ok(value) => match value.as_bool() {
            Some(bool) => bool,
            None => false,
        },
        Err(_) => false,
    }
}
