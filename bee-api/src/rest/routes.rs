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

use crate::api::{ApiImpl, Api};
use crate::rest::deserialize::deserialize_hash_array;

use bee_ternary::{T1B1Buf, TritBuf};
use bee_transaction::BundledTransaction;

use serde_json::Value;
use std::collections::HashMap;

pub async fn transaction_by_hash(json: Value) -> Result<impl warp::Reply, warp::Rejection> {

    match deserialize_hash_array(json["hashes"].as_array()) {

        Ok(hashes) => {

            let mut ret = HashMap::new();

            for (hash, tx_ref) in ApiImpl::transaction_by_hash(&hashes).iter() {

                let hash_string = hash.as_trits().
                    iter_trytes()
                    .map(|trit| char::from(trit))
                    .collect::<String>();

                match tx_ref {
                    Some(tx_ref) => {

                        let mut tx_buf = TritBuf::<T1B1Buf>::zeros(BundledTransaction::trit_len());
                        tx_ref.into_trits_allocated(&mut tx_buf);

                        let tx_string = tx_buf
                            .iter_trytes()
                            .map(|trit| char::from(trit))
                            .collect::<String>();

                        ret.insert(hash_string, Some(tx_string));
                    }
                    None => {
                        ret.insert(hash_string, None);
                    }
                }
            }

            Ok(warp::reply::json(&ret))

        }

        Err(x) => {
            Ok(warp::reply::json(&x.msg ))
        }

    }

}
