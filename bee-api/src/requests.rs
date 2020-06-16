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

use bee_transaction::{Hash, BundledTransactionField};
use std::convert::TryFrom;
use serde_json::Value as JsonValue;
use bee_ternary::{TryteBuf, T1B1Buf};

pub struct TransactionByHashRequest {
    pub hashes: Vec<Hash>
}

struct HashVecWrapper(pub Vec<Hash>);
struct HashWrapper(pub Hash);

impl TryFrom<&JsonValue> for TransactionByHashRequest {
    type Error = &'static str;
    fn try_from(value: &JsonValue) -> Result<Self, Self::Error> {
        Ok( TransactionByHashRequest { hashes: HashVecWrapper::try_from(value)?.0 } )
    }
}

impl TryFrom<&JsonValue> for HashVecWrapper {
    type Error = &'static str;
    fn try_from(value: &JsonValue) -> Result<Self, Self::Error> {
        match value["hashes"].as_array() {
            Some(hashes) => {
                let mut ret = Vec::new();
                for hash in hashes {
                    ret.push(HashWrapper::try_from(hash)?.0);
                }
                Ok(HashVecWrapper(ret))
            }
            None => Err("No hash array provided")
        }
    }
}

impl TryFrom<&JsonValue> for HashWrapper {
    type Error = &'static str;
    fn try_from(value: &JsonValue) -> Result<Self, Self::Error> {
        match value.as_str() {
            Some(tryte_str) => {
                match TryteBuf::try_from_str(tryte_str) {
                    Ok(buf) => {
                        let x = buf.as_trits().encode::<T1B1Buf>();
                        match Hash::try_from_inner(x) {
                            Ok(hash) => Ok(HashWrapper(hash)),
                            Err(_err) => Err("String has invalid size")
                        }
                    }
                    Err(_err) => Err("String contains invalid characters")
                }
            }
            None => Err("No string provided")
        }
    }

}