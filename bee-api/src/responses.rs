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

use bee_tangle::{MilestoneIndex, TransactionRef};
use bee_ternary::{T1B1Buf, TritBuf};
use bee_transaction::{BundledTransaction, Hash};

use serde_json::{Map, Value as JsonValue};

use std::{collections::HashMap, convert::From};

pub struct TransactionByHashResponse {
    pub hashes: HashMap<Hash, Option<TransactionRef>>,
}

impl From<TransactionByHashResponse> for JsonValue {
    fn from(res: TransactionByHashResponse) -> Self {
        let mut data = Map::new();

        for (hash, tx_ref) in res.hashes.iter() {
            let hash_string = hash
                .as_trits()
                .iter_trytes()
                .map(|trit| char::from(trit))
                .collect::<String>();

            match tx_ref {
                Some(tx_ref) => {
                    let mut tx_buf = TritBuf::<T1B1Buf>::zeros(BundledTransaction::trit_len());
                    tx_ref.into_trits_allocated(&mut tx_buf);

                    let tx_string = tx_buf.iter_trytes().map(|trit| char::from(trit)).collect::<String>();

                    data.insert(hash_string, JsonValue::String(tx_string));
                }
                None => {
                    data.insert(hash_string, JsonValue::Null);
                }
            }
        }

        json_success(data)
    }
}

pub struct NodeInfoResponse {
    pub is_synced: bool,
    pub last_milestone_index: MilestoneIndex,
    pub last_milestone_hash: Option<Hash>,
    pub last_solid_milestone_index: MilestoneIndex,
    pub last_solid_milestone_hash: Option<Hash>,
}

impl From<NodeInfoResponse> for JsonValue {
    fn from(res: NodeInfoResponse) -> Self {
        let mut data = Map::new();

        data.insert(String::from("is_synced"), JsonValue::Bool(res.is_synced));

        data.insert(
            String::from("last_milestone_index"),
            JsonValue::from(*res.last_milestone_index),
        );

        match res.last_milestone_hash {
            Some(hash) => {
                let hash_string = hash
                    .as_trits()
                    .iter_trytes()
                    .map(|trit| char::from(trit))
                    .collect::<String>();

                data.insert(String::from("last_milestone_hash"), JsonValue::String(hash_string));
            }
            None => {
                data.insert(String::from("last_milestone_hash"), JsonValue::Null);
            }
        }

        data.insert(
            String::from("last_solid_milestone_index"),
            JsonValue::from(*res.last_solid_milestone_index),
        );

        match res.last_solid_milestone_hash {
            Some(hash) => {
                let hash_string = hash
                    .as_trits()
                    .iter_trytes()
                    .map(|trit| char::from(trit))
                    .collect::<String>();

                data.insert(
                    String::from("last_solid_milestone_hash"),
                    JsonValue::String(hash_string),
                );
            }
            None => {
                data.insert(String::from("last_solid_milestone_hash"), JsonValue::Null);
            }
        }

        json_success(data)
    }
}

fn json_success(data: Map<String, JsonValue>) -> JsonValue {
    let mut response = Map::new();
    response.insert(String::from("data"), JsonValue::Object(data));
    JsonValue::Object(response)
}

pub fn json_error(msg: &str) -> JsonValue {
    let mut message = Map::new();
    message.insert(String::from("message"), JsonValue::String(String::from(msg)));
    let mut response = Map::new();
    response.insert(String::from("error"), JsonValue::Object(message));
    JsonValue::Object(response)
}
