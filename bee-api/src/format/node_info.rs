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

use crate::service::NodeInfoResponse;
use std::convert::From;

impl From<NodeInfoResponse> for JsonValue {
    fn from(res: NodeInfoResponse) -> Self {
        let mut json_obj = Map::new();

        json_obj.insert(String::from("is_synced"), JsonValue::from(&res.is_synced));

        json_obj.insert(
            String::from("last_milestone_index"),
            JsonValue::from(&res.last_milestone_index),
        );

        match res.last_milestone_hash {
            Some(hash) => json_obj.insert(String::from("last_milestone_hash"), JsonValue::from(&hash)),
            None => json_obj.insert(String::from("last_milestone_hash"), JsonValue::Null),
        };

        json_obj.insert(
            String::from("last_solid_milestone_index"),
            JsonValue::from(&res.last_solid_milestone_index),
        );

        match res.last_solid_milestone_hash {
            Some(hash) => json_obj.insert(String::from("last_solid_milestone_hash"), JsonValue::from(&hash)),
            None => json_obj.insert(String::from("last_solid_milestone_hash"), JsonValue::Null),
        };

        JsonValue::Object(json_obj)
    }
}
