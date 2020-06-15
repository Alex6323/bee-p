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

use bee_tangle::{
    tangle,
    TransactionRef
};
use bee_transaction::{Hash, TransactionVertex};
use std::collections::HashMap;

pub trait Api {

    fn approvers_of_transaction(hashes: &[Hash]) -> HashMap<Hash, Vec<TransactionRef>>;
    fn is_synced() -> bool;
    fn transaction_by_hash(hashes: &[Hash]) -> HashMap<Hash, Option<TransactionRef>>;

}

pub struct ApiImpl;

impl Api for ApiImpl {

    fn approvers_of_transaction(tx_hashes: &[Hash]) -> HashMap<Hash, Vec<TransactionRef>> {
        let mut ret = HashMap::new();
        for hash in tx_hashes {
            let mut without_hash = Vec::new();
            let approvers = tangle().trunk_walk_approvers(hash.clone(), |tx_ref| tx_ref.trunk().eq(hash) || tx_ref.branch().eq(hash));
            for (tx_ref, _hash)  in approvers {
                without_hash.push(tx_ref);
            }
            ret.insert(hash.clone(), without_hash);
        }
       ret
    }

    fn is_synced() -> bool {
        tangle().is_synced()
    }

    fn transaction_by_hash(tx_hashes: &[Hash]) -> HashMap<Hash, Option<TransactionRef>> {
        let mut ret = HashMap::new();
        for hash in tx_hashes {
            ret.insert(hash.clone(), tangle().get_transaction(&hash));
        }
        ret
    }

}