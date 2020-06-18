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

pub struct TransactionByHashRequest {
    pub hashes: Vec<Hash>,
}

struct DeserializedHashes(pub Vec<Hash>);
struct DeserializedHash(pub Hash);

impl TryFrom<&JsonValue> for TransactionByHashRequest {
    type Error = &'static str;
    fn try_from(value: &JsonValue) -> Result<Self, Self::Error> {
        match value["hashes"].as_array() {
            Some(hashes) => Ok(TransactionByHashRequest {
                hashes: DeserializedHashes::try_from(hashes)?.0,
            }),
            None => Err("No hash array provided"),
        }
    }
}

impl TryFrom<&Vec<JsonValue>> for DeserializedHashes {
    type Error = &'static str;
    fn try_from(value: &Vec<JsonValue>) -> Result<Self, Self::Error> {
        let mut ret = Vec::new();
        for hash in value {
            ret.push(DeserializedHash::try_from(hash)?.0);
        }
        Ok(DeserializedHashes(ret))
    }
}

impl TryFrom<&JsonValue> for DeserializedHash {
    type Error = &'static str;
    fn try_from(value: &JsonValue) -> Result<Self, Self::Error> {
        match value.as_str() {
            Some(tryte_str) => match TryteBuf::try_from_str(tryte_str) {
                Ok(buf) => {
                    let x = buf.as_trits().encode::<T1B1Buf>();
                    match Hash::try_from_inner(x) {
                        Ok(hash) => Ok(DeserializedHash(hash)),
                        Err(_err) => Err("String has invalid size"),
                    }
                }
                Err(_err) => Err("String contains invalid characters"),
            },
            None => Err("No string provided"),
        }
    }
}
