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

use crate::bundled::{
    constants::{ADDRESS, ESSENCE_TRIT_LEN, INDEX, IOTA_SUPPLY, OBSOLETE_TAG, TIMESTAMP, VALUE},
    Address, BundledTransaction, BundledTransactionError, BundledTransactionField, Index, Nonce, Payload, Tag,
    Timestamp, Value,
};

use bee_crypto::ternary::Hash;
use bee_ternary::{Btrit, T1B1Buf, TritBuf};

#[derive(Default)]
pub struct BundledTransactionBuilder {
    pub(crate) payload: Option<Payload>,
    pub(crate) address: Option<Address>,
    pub(crate) value: Option<Value>,
    pub(crate) obsolete_tag: Option<Tag>,
    pub(crate) timestamp: Option<Timestamp>,
    pub(crate) index: Option<Index>,
    pub(crate) last_index: Option<Index>,
    pub(crate) bundle: Option<Hash>,
    pub(crate) parent1: Option<Hash>,
    pub(crate) parent2: Option<Hash>,
    pub(crate) tag: Option<Tag>,
    pub(crate) attachment_ts: Option<Timestamp>,
    pub(crate) attachment_lbts: Option<Timestamp>,
    pub(crate) attachment_ubts: Option<Timestamp>,
    pub(crate) nonce: Option<Nonce>,
}

impl BundledTransactionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn essence(&self) -> TritBuf {
        let mut essence = TritBuf::<T1B1Buf>::zeros(ESSENCE_TRIT_LEN);

        let address = self.address.as_ref().unwrap();
        let value = TritBuf::<T1B1Buf<_>>::from(*self.value.as_ref().unwrap().to_inner());
        let obsolete_tag = self.obsolete_tag.as_ref().unwrap();
        let timestamp = TritBuf::<T1B1Buf<_>>::from(*self.timestamp.as_ref().unwrap().to_inner() as i128);
        let index = TritBuf::<T1B1Buf<_>>::from(*self.index.as_ref().unwrap().to_inner() as i128);
        let last_index = TritBuf::<T1B1Buf<_>>::from(*self.last_index.as_ref().unwrap().to_inner() as i128);

        let mut start = 0;
        let mut end = ADDRESS.trit_offset.length;

        essence[start..end].copy_from(address.to_inner());

        start += ADDRESS.trit_offset.length;
        end = start + value.len();
        essence[start..end].copy_from(&value);

        start += VALUE.trit_offset.length;
        end = start + OBSOLETE_TAG.trit_offset.length;
        essence[start..end].copy_from(obsolete_tag.to_inner());

        start += OBSOLETE_TAG.trit_offset.length;
        end = start + timestamp.len();
        essence[start..end].copy_from(&timestamp);

        start += TIMESTAMP.trit_offset.length;
        end = start + index.len();
        essence[start..end].copy_from(&index);

        start += INDEX.trit_offset.length;
        end = start + last_index.len();
        essence[start..end].copy_from(&last_index);

        essence
    }

    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload.replace(payload);
        self
    }

    pub fn with_address(mut self, address: Address) -> Self {
        self.address.replace(address);
        self
    }

    pub fn with_value(mut self, value: Value) -> Self {
        self.value.replace(value);
        self
    }

    pub fn with_obsolete_tag(mut self, obsolete_tag: Tag) -> Self {
        self.obsolete_tag.replace(obsolete_tag);
        self
    }
    pub fn with_timestamp(mut self, timestamp: Timestamp) -> Self {
        self.timestamp.replace(timestamp);
        self
    }

    pub fn with_index(mut self, index: Index) -> Self {
        self.index.replace(index);
        self
    }

    pub fn with_last_index(mut self, last_index: Index) -> Self {
        self.last_index.replace(last_index);
        self
    }

    pub fn with_tag(mut self, tag: Tag) -> Self {
        self.tag.replace(tag);
        self
    }

    pub fn with_attachment_ts(mut self, attachment_ts: Timestamp) -> Self {
        self.attachment_ts.replace(attachment_ts);
        self
    }

    pub fn with_bundle(mut self, bundle: Hash) -> Self {
        self.bundle.replace(bundle);
        self
    }

    pub fn with_parent1(mut self, parent1: Hash) -> Self {
        self.parent1.replace(parent1);
        self
    }

    pub fn with_parent2(mut self, parent2: Hash) -> Self {
        self.parent2.replace(parent2);
        self
    }

    pub fn with_attachment_lbts(mut self, attachment_lbts: Timestamp) -> Self {
        self.attachment_lbts.replace(attachment_lbts);
        self
    }

    pub fn with_attachment_ubts(mut self, attachment_ubts: Timestamp) -> Self {
        self.attachment_ubts.replace(attachment_ubts);
        self
    }

    pub fn with_nonce(mut self, nonce: Nonce) -> Self {
        self.nonce.replace(nonce);
        self
    }

    pub fn build(self) -> Result<BundledTransaction, BundledTransactionError> {
        let value = self
            .value
            .as_ref()
            .ok_or(BundledTransactionError::MissingField("value"))?
            .0;
        let address = self.address.ok_or(BundledTransactionError::MissingField("address"))?;

        if value.abs() > IOTA_SUPPLY {
            return Err(BundledTransactionError::InvalidValue(value));
        }

        if value != 0 && address.to_inner().get(ADDRESS.trit_offset.length - 1).unwrap() != Btrit::Zero {
            return Err(BundledTransactionError::InvalidAddress);
        }

        Ok(BundledTransaction {
            payload: self.payload.ok_or(BundledTransactionError::MissingField("payload"))?,
            address,
            value: Value(value),
            obsolete_tag: self
                .obsolete_tag
                .ok_or(BundledTransactionError::MissingField("obsolete_tag"))?,
            timestamp: self
                .timestamp
                .ok_or(BundledTransactionError::MissingField("timestamp"))?,
            index: self.index.ok_or(BundledTransactionError::MissingField("index"))?,
            last_index: self
                .last_index
                .ok_or(BundledTransactionError::MissingField("last_index"))?,
            tag: self.tag.ok_or(BundledTransactionError::MissingField("tag"))?,
            bundle: self.bundle.ok_or(BundledTransactionError::MissingField("bundle"))?,
            parent1: self.parent1.ok_or(BundledTransactionError::MissingField("parent1"))?,
            parent2: self.parent2.ok_or(BundledTransactionError::MissingField("parent2"))?,
            attachment_ts: self
                .attachment_ts
                .ok_or(BundledTransactionError::MissingField("attachment_ts"))?,
            attachment_lbts: self
                .attachment_lbts
                .ok_or(BundledTransactionError::MissingField("attachment_lbts"))?,
            attachment_ubts: self
                .attachment_ubts
                .ok_or(BundledTransactionError::MissingField("attachment_ubts"))?,
            nonce: self.nonce.ok_or(BundledTransactionError::MissingField("nonce"))?,
        })
    }
}

#[derive(Default)]
pub struct BundledTransactionBuilders(pub(crate) Vec<BundledTransactionBuilder>);

impl BundledTransactionBuilders {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn push(&mut self, transaction_builder: BundledTransactionBuilder) {
        self.0.push(transaction_builder);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::bundled::constants::TRANSACTION_TRIT_LEN;

    use bee_ternary::{Trits, T1B1};

    #[test]
    fn create_transaction_from_builder() {
        let _ = BundledTransactionBuilder::new()
            .with_payload(Payload::zeros())
            .with_address(Address::zeros())
            .with_value(Value(0))
            .with_obsolete_tag(Tag::zeros())
            .with_timestamp(Timestamp(0))
            .with_index(Index(0))
            .with_last_index(Index(0))
            .with_tag(Tag::zeros())
            .with_attachment_ts(Timestamp(0))
            .with_bundle(Hash::zeros())
            .with_parent1(Hash::zeros())
            .with_parent2(Hash::zeros())
            .with_attachment_lbts(Timestamp(0))
            .with_attachment_ubts(Timestamp(0))
            .with_nonce(Nonce::zeros())
            .build()
            .unwrap();
    }

    #[test]
    fn test_from_and_into_trits() {
        let tx = BundledTransactionBuilder::new()
            .with_payload(Payload::zeros())
            .with_address(Address::zeros())
            .with_value(Value(0))
            .with_obsolete_tag(Tag::zeros())
            .with_timestamp(Timestamp(0))
            .with_index(Index(0))
            .with_last_index(Index(0))
            .with_tag(Tag::zeros())
            .with_attachment_ts(Timestamp(0))
            .with_bundle(Hash::zeros())
            .with_parent1(Hash::zeros())
            .with_parent2(Hash::zeros())
            .with_attachment_lbts(Timestamp(0))
            .with_attachment_ubts(Timestamp(0))
            .with_nonce(Nonce::zeros())
            .build()
            .unwrap();

        let raw_tx_bytes: &mut [i8] = &mut [0 as i8; TRANSACTION_TRIT_LEN];
        let tx_trits = unsafe { Trits::<T1B1>::from_raw_unchecked_mut(raw_tx_bytes, TRANSACTION_TRIT_LEN) };

        tx.as_trits_allocated(tx_trits);
        let tx2 = BundledTransaction::from_trits(tx_trits).unwrap();

        assert_eq!(tx.payload, tx2.payload);
        assert_eq!(tx.address, tx2.address);
        assert_eq!(tx.value, tx2.value);
        assert_eq!(tx.obsolete_tag, tx2.obsolete_tag);
        assert_eq!(tx.timestamp, tx2.timestamp);
        assert_eq!(tx.index, tx2.index);
        assert_eq!(tx.last_index, tx2.last_index);
        assert_eq!(tx.tag, tx2.tag);
        assert_eq!(tx.attachment_ts, tx2.attachment_ts);
        assert_eq!(tx.bundle, tx2.bundle);
        assert_eq!(tx.parent1, tx2.parent1);
        assert_eq!(tx.parent2, tx2.parent2);
        assert_eq!(tx.attachment_lbts, tx2.attachment_lbts);
        assert_eq!(tx.attachment_ubts, tx2.attachment_ubts);
        assert_eq!(tx.nonce, tx2.nonce);
    }
}
