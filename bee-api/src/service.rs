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

use bee_tangle::{tangle, MilestoneIndex, TransactionRef};
use bee_transaction::Hash;

use std::collections::HashMap;

pub trait Service {
    fn node_info() -> NodeInfoResponse;
    fn transaction_by_hash(params: TransactionByHashParams) -> TransactionByHashResponse;
}

pub struct NodeInfoResponse {
    pub is_synced: bool,
    pub last_milestone_index: MilestoneIndex,
    pub last_milestone_hash: Option<Hash>,
    pub last_solid_milestone_index: MilestoneIndex,
    pub last_solid_milestone_hash: Option<Hash>,
}

pub struct TransactionByHashParams {
    pub hashes: Vec<Hash>,
}

pub struct TransactionByHashResponse {
    pub hashes: HashMap<Hash, Option<TransactionRef>>,
}

pub struct ServiceImpl;
impl Service for ServiceImpl {
    fn node_info() -> NodeInfoResponse {
        NodeInfoResponse {
            is_synced: tangle().is_synced(),
            last_milestone_index: tangle().get_last_milestone_index(),
            last_milestone_hash: tangle().get_milestone_hash(tangle().get_last_milestone_index()),
            last_solid_milestone_index: tangle().get_solid_milestone_index(),
            last_solid_milestone_hash: tangle().get_milestone_hash(tangle().get_solid_milestone_index()),
        }
    }

    fn transaction_by_hash(params: TransactionByHashParams) -> TransactionByHashResponse {
        let mut ret = HashMap::new();
        for hash in params.hashes {
            ret.insert(hash, tangle().get_transaction(&hash));
        }
        TransactionByHashResponse { hashes: ret }
    }
}
