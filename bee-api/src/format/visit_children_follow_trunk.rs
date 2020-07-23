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

use serde_json::Value as JsonValue;

use crate::{
    format::items::hash::HashItem,
    service::{VisitChildrenFollowTrunkParams, VisitChildrenFollowTrunkResponse},
};
use std::convert::{From, TryFrom};

impl TryFrom<&JsonValue> for VisitChildrenFollowTrunkParams {
    type Error = &'static str;
    fn try_from(value: &JsonValue) -> Result<Self, Self::Error> {
        let entry = match value["entry"].as_str() {
            Some(str) => HashItem::try_from(str)?.0,
            None => return Err("can not find initial hash"),
        };
        let traverse_cond = value["traverse_cond"].to_owned();
        let catch_cond = value["catch_cond"].to_owned();

        Ok(VisitChildrenFollowTrunkParams {
            entry,
            traverse_cond,
            catch_cond,
        })
    }
}

impl From<VisitChildrenFollowTrunkResponse> for JsonValue {
    fn from(res: VisitChildrenFollowTrunkResponse) -> Self {
        let mut hashes = Vec::new();
        for hash in res.tx_refs.iter() {
            hashes.push(JsonValue::from(&HashItem(hash.clone())));
        }
        JsonValue::Array(hashes)
    }
}
