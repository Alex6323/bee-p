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

use bee_tangle::tangle;

use crate::format::items::{
    bool_item::BoolItem, hash_item::HashItem, milestone_index_item::MilestoneIndexItem,
    transaction_ref_item::TransactionRefItem,
};
use std::collections::HashMap;

pub trait Service {
    fn node_info() -> NodeInfoResponse;
    fn transaction_by_hash(params: TransactionByHashParams) -> TransactionByHashResponse;
    fn transactions_by_hashes(params: TransactionsByHashesParams) -> TransactionsByHashesResponse;
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
    pub tx: Option<TransactionRefItem>,
}

pub struct TransactionsByHashesParams {
    pub hashes: Vec<HashItem>,
}

pub struct TransactionsByHashesResponse {
    pub txs: HashMap<HashItem, Option<TransactionRefItem>>,
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

    fn transaction_by_hash(params: TransactionByHashParams) -> TransactionByHashResponse {
        let ret = match tangle().get_transaction(&params.hash.0) {
            Some(tx_ref) => Some(TransactionRefItem(tx_ref)),
            None => None,
        };
        TransactionByHashResponse { tx: ret }
    }

    fn transactions_by_hashes(params: TransactionsByHashesParams) -> TransactionsByHashesResponse {
        let mut ret = HashMap::new();
        for hash in params.hashes {
            ret.insert(
                HashItem(hash.0),
                ServiceImpl::transaction_by_hash(TransactionByHashParams { hash }).tx,
            );
        }
        TransactionsByHashesResponse { txs: ret }
    }
}
