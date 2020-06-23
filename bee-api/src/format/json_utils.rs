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

pub fn json_success_obj(data: JsonValue) -> JsonValue {
    let mut response = Map::new();
    response.insert(String::from("data"), data);
    JsonValue::Object(response)
}

pub fn json_error_obj(msg: &str) -> JsonValue {
    let mut message = Map::new();
    message.insert(String::from("message"), JsonValue::String(String::from(msg)));
    let mut response = Map::new();
    response.insert(String::from("error"), JsonValue::Object(message));
    JsonValue::Object(response)
}
