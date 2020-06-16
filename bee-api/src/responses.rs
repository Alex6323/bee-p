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

use std::collections::HashMap;
use bee_transaction::{Hash, BundledTransaction};
use bee_tangle::TransactionRef;
use std::convert::From;
use serde_json::{Value as JsonValue, Map};
use bee_ternary::{TritBuf, T1B1Buf};

pub struct TransactionByHashResponse {
    pub hashes: HashMap<Hash, Option<TransactionRef>>
}

impl From<TransactionByHashResponse> for JsonValue {

    fn from(res: TransactionByHashResponse) -> Self {

        let mut map = Map::new();

        for (hash, tx_ref) in res.hashes.iter() {

            let hash_string = hash.as_trits().
                iter_trytes()
                .map(|trit| char::from(trit))
                .collect::<String>();

            match tx_ref {
                Some(tx_ref) => {

                    let mut tx_buf = TritBuf::<T1B1Buf>::zeros(BundledTransaction::trit_len());
                    tx_ref.into_trits_allocated(&mut tx_buf);

                    let tx_string = tx_buf
                        .iter_trytes()
                        .map(|trit| char::from(trit))
                        .collect::<String>();

                    map.insert(hash_string, JsonValue::String(tx_string));
                }
                None => {
                    map.insert(hash_string, JsonValue::Null);
                }
            }

        }

       JsonValue::Object(map)

    }

}

pub struct NodeInfoResponse {
    pub is_synced: bool
}

impl From<NodeInfoResponse> for JsonValue {

    fn from(res: NodeInfoResponse) -> Self {
        let mut map = Map::new();
        map.insert(String::from("is_synced"), JsonValue::Bool(res.is_synced));
        JsonValue::Object(map)
    }

}