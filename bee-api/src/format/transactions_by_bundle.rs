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
    service::{TransactionsByBundleParams, TransactionsByBundleResponse},
};
use std::convert::{From, TryFrom};

impl TryFrom<&JsonValue> for TransactionsByBundleParams {
    type Error = &'static str;
    fn try_from(value: &JsonValue) -> Result<Self, Self::Error> {
        let entry = match value["entry"].as_str() {
            Some(str) => HashItem::try_from(str)?,
            None => return Err("Can not find entry hash")
        };
        let bundle = match value["bundle"].as_str() {
            Some(str) => HashItem::try_from(str)?,
            None => return Err("Can not find bundle hash")
        };
        Ok(TransactionsByBundleParams {
            entry,
            bundle,
        })
    }
}

impl From<TransactionsByBundleResponse> for JsonValue {
    fn from(res: TransactionsByBundleResponse) -> Self {
        let mut json_obj = Map::new();
        for (hash, tx_ref) in res.tx_refs.iter() {
            json_obj.insert(String::from(hash), JsonValue::from(tx_ref));
        }
        JsonValue::Object(json_obj)
    }
}
