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

use serde_json::{Map, Value as JsonValue};

use crate::{
    format::items::hash::HashItem,
    service::{TransactionsByHashesParams, TransactionsByHashesResponse},
};
use std::convert::{From, TryFrom};
use crate::format::items::transaction_ref::TransactionRefItem;

impl TryFrom<&JsonValue> for TransactionsByHashesParams {
    type Error = &'static str;
    fn try_from(value: &JsonValue) -> Result<Self, Self::Error> {
        match value["hashes"].as_array() {
            Some(hashes) => {
                let mut ret = Vec::new();
                for value in hashes {
                    match value.as_str() {
                        Some(str) => ret.push(HashItem::try_from(str)?.0),
                        None => return Err("no string provided"),
                    }
                }
                Ok(TransactionsByHashesParams { hashes: ret })
            }
            None => Err("no hash array provided"),
        }
    }
}

impl From<TransactionsByHashesResponse> for JsonValue {
    fn from(res: TransactionsByHashesResponse) -> Self {
        let mut json_obj = Map::new();
        for (hash, tx_ref) in res.tx_refs.iter() {
            match tx_ref {
                Some(tx_ref) => json_obj.insert(String::from(&HashItem(hash.clone())), JsonValue::from(&TransactionRefItem(tx_ref.clone()))),
                None => json_obj.insert(String::from(&HashItem(hash.clone())), JsonValue::Null),
            };
        }
        JsonValue::Object(json_obj)
    }
}
