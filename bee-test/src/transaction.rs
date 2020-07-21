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

use crate::field::rand_trits_field;

use bee_crypto_ext::ternary::Hash;
use bee_transaction::{
    bundled::{
        Address, BundledTransaction as Transaction, BundledTransactionBuilder as TransactionBuilder,
        BundledTransactionField, Index, Nonce, Payload, Tag, Timestamp, Value,
    },
    TransactionVertex,
};

pub fn clone_tx(tx: &Transaction) -> Transaction {
    let builder = TransactionBuilder::new()
        .with_payload(tx.payload().clone())
        .with_address(tx.address().clone())
        .with_value(tx.value().clone())
        .with_obsolete_tag(tx.obsolete_tag().clone())
        .with_timestamp(tx.timestamp().clone())
        .with_index(tx.index().clone())
        .with_last_index(tx.last_index().clone())
        .with_tag(tx.tag().clone())
        .with_attachment_ts(tx.attachment_ts().clone())
        .with_bundle(tx.bundle().clone())
        .with_trunk(tx.trunk().clone())
        .with_branch(tx.branch().clone())
        .with_attachment_lbts(tx.attachment_lbts().clone())
        .with_attachment_ubts(tx.attachment_ubts().clone())
        .with_nonce(tx.nonce().clone());

    builder.build().unwrap()
}

pub fn create_random_tx() -> (Hash, Transaction) {
    let builder = TransactionBuilder::new()
        .with_payload(Payload::zeros())
        .with_address(rand_trits_field::<Address>())
        .with_value(Value::from_inner_unchecked(0))
        .with_obsolete_tag(rand_trits_field::<Tag>())
        .with_timestamp(Timestamp::from_inner_unchecked(0))
        .with_index(Index::from_inner_unchecked(0))
        .with_last_index(Index::from_inner_unchecked(0))
        .with_tag(rand_trits_field::<Tag>())
        .with_attachment_ts(Timestamp::from_inner_unchecked(0))
        .with_bundle(rand_trits_field::<Hash>())
        .with_trunk(rand_trits_field::<Hash>())
        .with_branch(rand_trits_field::<Hash>())
        .with_attachment_lbts(Timestamp::from_inner_unchecked(0))
        .with_attachment_ubts(Timestamp::from_inner_unchecked(0))
        .with_nonce(rand_trits_field::<Nonce>());

    (rand_trits_field::<Hash>(), builder.build().unwrap())
}

pub fn create_random_attached_tx(branch: Hash, trunk: Hash) -> (Hash, Transaction) {
    let builder = TransactionBuilder::new()
        .with_payload(rand_trits_field::<Payload>())
        .with_address(rand_trits_field::<Address>())
        .with_value(Value::from_inner_unchecked(0))
        .with_obsolete_tag(rand_trits_field::<Tag>())
        .with_timestamp(Timestamp::from_inner_unchecked(0))
        .with_index(Index::from_inner_unchecked(0))
        .with_last_index(Index::from_inner_unchecked(0))
        .with_tag(rand_trits_field::<Tag>())
        .with_attachment_ts(Timestamp::from_inner_unchecked(0))
        .with_bundle(rand_trits_field::<Hash>())
        .with_trunk(trunk)
        .with_branch(branch)
        .with_attachment_lbts(Timestamp::from_inner_unchecked(0))
        .with_attachment_ubts(Timestamp::from_inner_unchecked(0))
        .with_nonce(rand_trits_field::<Nonce>());

    (rand_trits_field::<Hash>(), builder.build().unwrap())
}
