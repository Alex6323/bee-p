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
    format::items::hash_item::HashItem,
    service::{TransactionByHashParams, TransactionByHashResponse},
};
use std::convert::{From, TryFrom};

impl TryFrom<&JsonValue> for TransactionByHashParams {
    type Error = &'static str;
    fn try_from(value: &JsonValue) -> Result<Self, Self::Error> {
        match value["hashes"].as_array() {
            Some(hashes) => {
                let mut ret = Vec::new();
                for value in hashes {
                    let hash_item = HashItem::try_from(value)?;
                    ret.push(hash_item);
                }
                Ok(TransactionByHashParams { hashes: ret })
            }
            None => Err("No hash array provided"),
        }
    }
}

impl From<TransactionByHashResponse> for JsonValue {
    fn from(res: TransactionByHashResponse) -> Self {
        let mut json_obj = Map::new();
        for (hash, tx_ref) in res.hashes.iter() {
            match tx_ref {
                Some(tx_ref) => json_obj.insert(String::from(hash), JsonValue::from(tx_ref)),
                None => json_obj.insert(String::from(hash), JsonValue::Null),
            };
        }
        JsonValue::Object(json_obj)
    }
}
