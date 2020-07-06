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

use crate::format::items::{
    bool::BoolItem, hash::HashItem, milestone_index::MilestoneIndexItem,
    transaction_ref::TransactionRefItem,
};

use bee_protocol::tangle::tangle;
use bee_tangle::traversal;
use std::collections::HashMap;
use std::fmt;

pub trait Service {
    fn node_info() -> NodeInfoResponse;
    fn transactions_by_bundle(params: TransactionsByBundleParams) -> Result<TransactionsByBundleResponse, ServiceError>;
    fn transaction_by_hash(params: TransactionByHashParams) -> TransactionByHashResponse;
    fn transactions_by_hashes(params: TransactionsByHashesParams) -> TransactionsByHashesResponse;
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
    pub is_synced: BoolItem,
    pub last_milestone_index: MilestoneIndexItem,
    pub last_milestone_hash: Option<HashItem>,
    pub last_solid_milestone_index: MilestoneIndexItem,
    pub last_solid_milestone_hash: Option<HashItem>,
}

pub struct TransactionByHashParams {
    pub hash: HashItem,
}

pub struct TransactionByHashResponse {
    pub tx_ref: Option<TransactionRefItem>,
}

pub struct TransactionsByHashesParams {
    pub hashes: Vec<HashItem>,
}

pub struct TransactionsByHashesResponse {
    pub tx_refs: HashMap<HashItem, Option<TransactionRefItem>>,
}

pub struct TransactionsByBundleParams {
    pub entry: HashItem,
    pub bundle: HashItem,
}

pub struct TransactionsByBundleResponse {
    pub tx_refs: HashMap<HashItem, TransactionRefItem>,
}

pub struct ServiceImpl;
impl Service for ServiceImpl {
    fn node_info() -> NodeInfoResponse {
        let is_synced = BoolItem(tangle().is_synced());
        let last_milestone_index = MilestoneIndexItem(tangle().get_last_milestone_index());
        let last_milestone_hash = match tangle().get_milestone_hash(tangle().get_last_milestone_index()) {
            Some(hash) => Some(HashItem(hash)),
            None => None,
        };
        let last_solid_milestone_index = MilestoneIndexItem(tangle().get_solid_milestone_index());
        let last_solid_milestone_hash = match tangle().get_milestone_hash(tangle().get_solid_milestone_index()) {
            Some(hash) => Some(HashItem(hash)),
            None => None,
        };
        NodeInfoResponse {
            is_synced,
            last_milestone_index,
            last_milestone_hash,
            last_solid_milestone_index,
            last_solid_milestone_hash,
        }
    }

    fn transactions_by_bundle(params: TransactionsByBundleParams) -> Result<TransactionsByBundleResponse, ServiceError> {
        let mut ret = HashMap::new();
        if params.entry == params.bundle {
            return Err(ServiceError::new(String::from("entry hash is equal to bundle hash")))
        }
        traversal::visit_children_depth_first(
            tangle(),
            params.entry.0,
            |tx, _| tx.bundle() == &params.bundle.0,
            |tx_hash, tx, _| {
                ret.insert(HashItem(tx_hash.clone()), TransactionRefItem(tx.clone()));
            },
            |_| (),
        );
        Ok(TransactionsByBundleResponse { tx_refs: ret })
    }

    fn transaction_by_hash(params: TransactionByHashParams) -> TransactionByHashResponse {
        let ret = match tangle().get(&params.hash.0) {
            Some(tx_ref) => Some(TransactionRefItem(tx_ref)),
            None => None,
        };
        TransactionByHashResponse { tx_ref: ret }
    }

    fn transactions_by_hashes(params: TransactionsByHashesParams) -> TransactionsByHashesResponse {
        let mut ret = HashMap::new();
        for hash in params.hashes {
            ret.insert(
                HashItem(hash.0),
                ServiceImpl::transaction_by_hash(TransactionByHashParams { hash }).tx_ref,
            );
        }
        TransactionsByHashesResponse { tx_refs: ret }
    }

}
