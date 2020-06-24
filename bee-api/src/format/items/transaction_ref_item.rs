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

use bee_ternary::{T1B1Buf, TritBuf};

use bee_tangle::TransactionRef;
use serde_json::{Map, Value as JsonValue};

use bee_transaction::{
    bundled::{BundledTransaction, BundledTransactionField},
    TransactionVertex,
};

pub struct TransactionRefItem(pub TransactionRef);

impl From<&TransactionRefItem> for JsonValue {
    fn from(tx_ref: &TransactionRefItem) -> Self {
        let payload = tx_ref.0.payload();
        let address = tx_ref.0.address();
        let value = tx_ref.0.value();
        let obolete_tag = tx_ref.0.obsolete_tag();
        let timestamp = tx_ref.0.timestamp();
        let index = tx_ref.0.index();
        let last_index = tx_ref.0.last_index();
        let bundle = tx_ref.0.bundle();
        let trunk = tx_ref.0.trunk();
        let branch = tx_ref.0.branch();
        let tag = tx_ref.0.tag();
        let attachment_ts = tx_ref.0.attachment_ts();
        let attachment_lbts = tx_ref.0.attachment_lbts();
        let attachment_ubts = tx_ref.0.attachment_ubts();
        let nonce = tx_ref.0.nonce();

        let mut json_obj = Map::new();

        json_obj.insert(
            String::from("payload"),
            JsonValue::String(
                payload
                    .to_inner()
                    .iter_trytes()
                    .map(|trit| char::from(trit))
                    .collect::<String>(),
            ),
        );
        json_obj.insert(
            String::from("address"),
            JsonValue::String(
                address
                    .to_inner()
                    .iter_trytes()
                    .map(|trit| char::from(trit))
                    .collect::<String>(),
            ),
        );
        json_obj.insert(String::from("value"), JsonValue::from(value.to_inner().clone()));
        json_obj.insert(
            String::from("obsolete_tag"),
            JsonValue::String(
                obolete_tag
                    .to_inner()
                    .iter_trytes()
                    .map(|trit| char::from(trit))
                    .collect::<String>(),
            ),
        );
        json_obj.insert(String::from("timestamp"), JsonValue::from(timestamp.to_inner().clone()));
        json_obj.insert(String::from("index"), JsonValue::from(index.to_inner().clone()));
        json_obj.insert(
            String::from("last_index"),
            JsonValue::from(last_index.to_inner().clone()),
        );
        json_obj.insert(
            String::from("bundle"),
            JsonValue::String(
                bundle
                    .to_inner()
                    .iter_trytes()
                    .map(|trit| char::from(trit))
                    .collect::<String>(),
            ),
        );
        json_obj.insert(
            String::from("trunk"),
            JsonValue::String(
                trunk
                    .to_inner()
                    .iter_trytes()
                    .map(|trit| char::from(trit))
                    .collect::<String>(),
            ),
        );
        json_obj.insert(
            String::from("branch"),
            JsonValue::String(
                branch
                    .to_inner()
                    .iter_trytes()
                    .map(|trit| char::from(trit))
                    .collect::<String>(),
            ),
        );
        json_obj.insert(
            String::from("tag"),
            JsonValue::String(
                tag.to_inner()
                    .iter_trytes()
                    .map(|trit| char::from(trit))
                    .collect::<String>(),
            ),
        );
        json_obj.insert(
            String::from("attachment_ts"),
            JsonValue::from(attachment_ts.to_inner().clone()),
        );
        json_obj.insert(
            String::from("attachment_lbts"),
            JsonValue::from(attachment_lbts.to_inner().clone()),
        );
        json_obj.insert(
            String::from("attachment_ubts"),
            JsonValue::from(attachment_ubts.to_inner().clone()),
        );
        json_obj.insert(
            String::from("nonce"),
            JsonValue::String(
                nonce
                    .to_inner()
                    .iter_trytes()
                    .map(|trit| char::from(trit))
                    .collect::<String>(),
            ),
        );

        JsonValue::Object(json_obj)
    }
}

impl From<&TransactionRefItem> for String {
    fn from(value: &TransactionRefItem) -> Self {
        let mut tx_buf = TritBuf::<T1B1Buf>::zeros(BundledTransaction::trit_len());
        value.0.into_trits_allocated(&mut tx_buf);
        tx_buf.iter_trytes().map(|trit| char::from(trit)).collect::<String>()
    }
}
