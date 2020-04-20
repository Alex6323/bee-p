use crate::{
    constants::{
        ADDRESS,
        IOTA_SUPPLY,
    },
    transaction::{
        Address,
        Hash,
        Index,
        Nonce,
        Payload,
        Tag,
        Timestamp,
        Transaction,
        TransactionError,
        TransactionField,
        Value,
    },
};

use bee_ternary::Btrit;

#[derive(Default)]
pub struct TransactionBuilder {
    pub(crate) payload: Option<Payload>,
    pub(crate) address: Option<Address>,
    pub(crate) value: Option<Value>,
    pub(crate) obsolete_tag: Option<Tag>,
    pub(crate) timestamp: Option<Timestamp>,
    pub(crate) index: Option<Index>,
    pub(crate) last_index: Option<Index>,
    pub(crate) bundle: Option<Hash>,
    pub(crate) trunk: Option<Hash>,
    pub(crate) branch: Option<Hash>,
    pub(crate) tag: Option<Tag>,
    pub(crate) attachment_ts: Option<Timestamp>,
    pub(crate) attachment_lbts: Option<Timestamp>,
    pub(crate) attachment_ubts: Option<Timestamp>,
    pub(crate) nonce: Option<Nonce>,
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self::default()
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

    pub fn with_trunk(mut self, trunk: Hash) -> Self {
        self.trunk.replace(trunk);
        self
    }

    pub fn with_branch(mut self, branch: Hash) -> Self {
        self.branch.replace(branch);
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

    pub fn build(self) -> Result<Transaction, TransactionError> {
        let value = self.value.as_ref().ok_or(TransactionError::MissingField("value"))?.0;
        let address = self.address.ok_or(TransactionError::MissingField("address"))?;

        if value.abs() > IOTA_SUPPLY {
            Err(TransactionError::InvalidValue(value))?;
        }

        if value != 0 && address.to_inner().get(ADDRESS.trit_offset.length - 1).unwrap() != Btrit::Zero {
            Err(TransactionError::InvalidAddress)?;
        }

        Ok(Transaction {
            payload: self.payload.ok_or(TransactionError::MissingField("payload"))?,
            address,
            value: Value(value),
            obsolete_tag: self
                .obsolete_tag
                .ok_or(TransactionError::MissingField("obsolete_tag"))?,
            timestamp: self.timestamp.ok_or(TransactionError::MissingField("timestamp"))?,
            index: self.index.ok_or(TransactionError::MissingField("index"))?,
            last_index: self.last_index.ok_or(TransactionError::MissingField("last_index"))?,
            tag: self.tag.ok_or(TransactionError::MissingField("tag"))?,
            bundle: self.bundle.ok_or(TransactionError::MissingField("bundle"))?,
            trunk: self.trunk.ok_or(TransactionError::MissingField("trunk"))?,
            branch: self.branch.ok_or(TransactionError::MissingField("branch"))?,
            attachment_ts: self
                .attachment_ts
                .ok_or(TransactionError::MissingField("attachment_ts"))?,
            attachment_lbts: self
                .attachment_lbts
                .ok_or(TransactionError::MissingField("attachment_lbts"))?,
            attachment_ubts: self
                .attachment_ubts
                .ok_or(TransactionError::MissingField("attachment_ubts"))?,
            nonce: self.nonce.ok_or(TransactionError::MissingField("nonce"))?,
        })
    }
}

#[derive(Default)]
pub struct TransactionBuilders(pub(crate) Vec<TransactionBuilder>);

impl TransactionBuilders {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push(&mut self, transaction_builder: TransactionBuilder) {
        self.0.push(transaction_builder);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::constants::TRANSACTION_TRIT_LEN;

    use bee_ternary::{
        Trits,
        T1B1,
    };

    #[test]
    fn create_transaction_from_builder() {
        let _ = TransactionBuilder::new()
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
            .with_trunk(Hash::zeros())
            .with_branch(Hash::zeros())
            .with_attachment_lbts(Timestamp(0))
            .with_attachment_ubts(Timestamp(0))
            .with_nonce(Nonce::zeros())
            .build()
            .unwrap();
    }

    #[test]
    fn test_from_and_into_trits() {
        let tx = TransactionBuilder::new()
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
            .with_trunk(Hash::zeros())
            .with_branch(Hash::zeros())
            .with_attachment_lbts(Timestamp(0))
            .with_attachment_ubts(Timestamp(0))
            .with_nonce(Nonce::zeros())
            .build()
            .unwrap();

        let raw_tx_bytes: &mut [i8] = &mut [0 as i8; TRANSACTION_TRIT_LEN];
        let tx_trits = unsafe { Trits::<T1B1>::from_raw_unchecked_mut(raw_tx_bytes, TRANSACTION_TRIT_LEN) };

        tx.into_trits_allocated(tx_trits);
        let tx2 = Transaction::from_trits(tx_trits).unwrap();

        assert_eq!(tx.payload, tx2.payload);
        assert_eq!(tx.bundle, tx2.bundle);
        assert_eq!(tx.trunk, tx2.trunk);
        assert_eq!(tx.branch, tx2.branch);
        assert_eq!(tx.nonce, tx2.nonce);
        assert_eq!(tx.tag, tx2.tag);
        assert_eq!(tx.obsolete_tag, tx2.obsolete_tag);
        assert_eq!(tx.value, tx2.value);
        assert_eq!(tx.timestamp, tx2.timestamp);
        assert_eq!(tx.attachment_ts, tx2.attachment_ts);
        assert_eq!(tx.attachment_ubts, tx2.attachment_ubts);
        assert_eq!(tx.index, tx2.index);
        assert_eq!(tx.last_index, tx2.last_index);
    }
}
