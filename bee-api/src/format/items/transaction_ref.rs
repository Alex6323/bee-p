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

use bee_tangle::TransactionRef;
use serde_json::{Map, Value as JsonValue};

use bee_transaction::{bundled::BundledTransactionField, TransactionVertex};

pub struct TransactionRefItem(pub TransactionRef);

impl From<&TransactionRefItem> for JsonValue {
    fn from(tx_ref: &TransactionRefItem) -> Self {
        let mut json_obj = Map::new();

        json_obj.insert(
            String::from("payload"),
            JsonValue::String(
                tx_ref
                    .0
                    .payload()
                    .to_inner()
                    .iter_trytes()
                    .map(|trit| char::from(trit))
                    .collect::<String>(),
            ),
        );
        json_obj.insert(
            String::from("address"),
            JsonValue::String(
                tx_ref
                    .0
                    .address()
                    .to_inner()
                    .iter_trytes()
                    .map(|trit| char::from(trit))
                    .collect::<String>(),
            ),
        );
        json_obj.insert(
            String::from("value"),
            JsonValue::from(tx_ref.0.value().to_inner().clone()),
        );
        json_obj.insert(
            String::from("obsolete_tag"),
            JsonValue::String(
                tx_ref
                    .0
                    .obsolete_tag()
                    .to_inner()
                    .iter_trytes()
                    .map(|trit| char::from(trit))
                    .collect::<String>(),
            ),
        );
        json_obj.insert(
            String::from("timestamp"),
            JsonValue::from(tx_ref.0.timestamp().to_inner().clone()),
        );
        json_obj.insert(
            String::from("index"),
            JsonValue::from(tx_ref.0.index().to_inner().clone()),
        );
        json_obj.insert(
            String::from("last_index"),
            JsonValue::from(tx_ref.0.last_index().to_inner().clone()),
        );
        json_obj.insert(
            String::from("bundle"),
            JsonValue::String(
                tx_ref
                    .0
                    .bundle()
                    .to_inner()
                    .iter_trytes()
                    .map(|trit| char::from(trit))
                    .collect::<String>(),
            ),
        );
        json_obj.insert(
            String::from("trunk"),
            JsonValue::String(
                tx_ref
                    .0
                    .trunk()
                    .to_inner()
                    .iter_trytes()
                    .map(|trit| char::from(trit))
                    .collect::<String>(),
            ),
        );
        json_obj.insert(
            String::from("branch"),
            JsonValue::String(
                tx_ref
                    .0
                    .branch()
                    .to_inner()
                    .iter_trytes()
                    .map(|trit| char::from(trit))
                    .collect::<String>(),
            ),
        );
        json_obj.insert(
            String::from("tag"),
            JsonValue::String(
                tx_ref
                    .0
                    .tag()
                    .to_inner()
                    .iter_trytes()
                    .map(|trit| char::from(trit))
                    .collect::<String>(),
            ),
        );
        json_obj.insert(
            String::from("attachment_ts"),
            JsonValue::from(tx_ref.0.attachment_ts().to_inner().clone()),
        );
        json_obj.insert(
            String::from("attachment_lbts"),
            JsonValue::from(tx_ref.0.attachment_lbts().to_inner().clone()),
        );
        json_obj.insert(
            String::from("attachment_ubts"),
            JsonValue::from(tx_ref.0.attachment_ubts().to_inner().clone()),
        );
        json_obj.insert(
            String::from("nonce"),
            JsonValue::String(
                tx_ref
                    .0
                    .nonce()
                    .to_inner()
                    .iter_trytes()
                    .map(|trit| char::from(trit))
                    .collect::<String>(),
            ),
        );

        JsonValue::Object(json_obj)
    }
}
