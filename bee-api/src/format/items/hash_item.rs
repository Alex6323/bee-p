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

use bee_ternary::{T1B1Buf, TryteBuf};
use bee_transaction::{BundledTransactionField, Hash};

use serde_json::Value as JsonValue;

use std::convert::TryFrom;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct HashItem(pub Hash);

impl TryFrom<&str> for HashItem {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match TryteBuf::try_from_str(value) {
            Ok(buf) => match Hash::try_from_inner(buf.as_trits().encode::<T1B1Buf>()) {
                Ok(hash) => Ok(HashItem(hash)),
                Err(_err) => Err("String has invalid size"),
            },
            Err(_err) => Err("String contains invalid characters"),
        }
    }
}

impl TryFrom<&JsonValue> for HashItem {
    type Error = &'static str;
    fn try_from(value: &JsonValue) -> Result<Self, Self::Error> {
        match value.as_str() {
            Some(tryte_str) => HashItem::try_from(tryte_str),
            None => Err("No string provided"),
        }
    }
}

impl From<&HashItem> for String {
    fn from(value: &HashItem) -> Self {
        value
            .0
            .as_trits()
            .iter_trytes()
            .map(|trit| char::from(trit))
            .collect::<String>()
    }
}

impl From<&HashItem> for JsonValue {
    fn from(value: &HashItem) -> Self {
        JsonValue::String(String::from(value))
    }
}
