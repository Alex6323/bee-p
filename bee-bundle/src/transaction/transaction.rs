use crate::{
    constants::{
        ADDRESS,
        ATTACHMENT_LBTS,
        ATTACHMENT_TS,
        ATTACHMENT_UBTS,
        BRANCH,
        BUNDLE,
        INDEX,
        IOTA_SUPPLY,
        LAST_INDEX,
        NONCE,
        OBSOLETE_TAG,
        PAYLOAD,
        TAG,
        TIMESTAMP,
        TRANSACTION_TRIT_LEN,
        TRUNK,
        VALUE,
    },
    transaction::{
        Address,
        Hash,
        Index,
        Nonce,
        Payload,
        Tag,
        Timestamp,
        TransactionField,
        Value,
    },
};

use bee_ternary::{
    num_conversions,
    raw::RawEncoding,
    Btrit,
    T1B1Buf,
    TritBuf,
    Trits,
    T1B1,
};

use std::convert::TryFrom;

#[derive(Debug)]
pub enum TransactionError {
    TransactionDeserializationError,
    TransactionInvalidValue,
    TransactionBuilderError(TransactionBuilderError),
}

impl From<num_conversions::TritsI64ConversionError> for TransactionError {
    fn from(_: num_conversions::TritsI64ConversionError) -> Self {
        TransactionError::TransactionInvalidValue
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Transaction {
    pub(crate) payload: Payload,
    pub(crate) address: Address,
    pub(crate) value: Value,
    pub(crate) obsolete_tag: Tag,
    pub(crate) timestamp: Timestamp,
    pub(crate) index: Index,
    pub(crate) last_index: Index,
    pub(crate) bundle: Hash,
    pub(crate) trunk: Hash,
    pub(crate) branch: Hash,
    pub(crate) tag: Tag,
    pub(crate) attachment_ts: Timestamp,
    pub(crate) attachment_lbts: Timestamp,
    pub(crate) attachment_ubts: Timestamp,
    pub(crate) nonce: Nonce,
}

impl Eq for Transaction {}

impl Transaction {
    pub fn builder() -> TransactionBuilder {
        TransactionBuilder::new()
    }

    pub fn from_trits(buffer: &Trits<impl RawEncoding<Trit = Btrit> + ?Sized>) -> Result<Self, TransactionError> {
        let trits = buffer.encode::<T1B1Buf>();

        let value =
            i64::try_from(trits[VALUE.trit_offset.start..VALUE.trit_offset.start + VALUE.trit_offset.length].to_buf())?;

        let timestamp = i64::try_from(
            trits[TIMESTAMP.trit_offset.start..TIMESTAMP.trit_offset.start + TIMESTAMP.trit_offset.length].to_buf(),
        )? as u64;
        let index =
            i64::try_from(trits[INDEX.trit_offset.start..INDEX.trit_offset.start + INDEX.trit_offset.length].to_buf())?
                as usize;
        let last_index = i64::try_from(
            trits[LAST_INDEX.trit_offset.start..LAST_INDEX.trit_offset.start + LAST_INDEX.trit_offset.length].to_buf(),
        )? as usize;

        let attachment_ts = i64::try_from(
            trits[ATTACHMENT_TS.trit_offset.start..ATTACHMENT_TS.trit_offset.start + ATTACHMENT_TS.trit_offset.length]
                .to_buf(),
        )? as u64;

        let attachment_lbts = i64::try_from(
            trits[ATTACHMENT_LBTS.trit_offset.start
                ..ATTACHMENT_LBTS.trit_offset.start + ATTACHMENT_LBTS.trit_offset.length]
                .to_buf(),
        )? as u64;
        let attachment_ubts = i64::try_from(
            trits[ATTACHMENT_UBTS.trit_offset.start
                ..ATTACHMENT_UBTS.trit_offset.start + ATTACHMENT_UBTS.trit_offset.length]
                .to_buf(),
        )? as u64;

        let transaction = Self::builder()
            .with_payload(Payload(
                trits[PAYLOAD.trit_offset.start..PAYLOAD.trit_offset.start + PAYLOAD.trit_offset.length].to_buf(),
            ))
            .with_address(Address(
                trits[ADDRESS.trit_offset.start..ADDRESS.trit_offset.start + ADDRESS.trit_offset.length].to_buf(),
            ))
            .with_value(Value::from_inner_unchecked(value))
            .with_obsolete_tag(Tag(trits[OBSOLETE_TAG.trit_offset.start
                ..OBSOLETE_TAG.trit_offset.start + OBSOLETE_TAG.trit_offset.length]
                .to_buf()))
            .with_timestamp(Timestamp::from_inner_unchecked(timestamp))
            .with_index(Index::from_inner_unchecked(index))
            .with_last_index(Index::from_inner_unchecked(last_index))
            .with_tag(Tag(trits
                [TAG.trit_offset.start..TAG.trit_offset.start + TAG.trit_offset.length]
                .to_buf()))
            .with_attachment_ts(Timestamp::from_inner_unchecked(attachment_ts))
            .with_bundle(Hash::from_inner_unchecked(
                trits[BUNDLE.trit_offset.start..BUNDLE.trit_offset.start + BUNDLE.trit_offset.length].to_buf(),
            ))
            .with_trunk(Hash::from_inner_unchecked(
                trits[TRUNK.trit_offset.start..TRUNK.trit_offset.start + TRUNK.trit_offset.length].to_buf(),
            ))
            .with_branch(Hash::from_inner_unchecked(
                trits[BRANCH.trit_offset.start..BRANCH.trit_offset.start + BRANCH.trit_offset.length].to_buf(),
            ))
            .with_attachment_lbts(Timestamp::from_inner_unchecked(attachment_lbts))
            .with_attachment_ubts(Timestamp::from_inner_unchecked(attachment_ubts))
            .with_nonce(Nonce(
                trits[NONCE.trit_offset.start..NONCE.trit_offset.start + NONCE.trit_offset.length].to_buf(),
            ))
            .build()
            .map_err(|e| TransactionError::TransactionBuilderError(e))?;

        Ok(transaction)
    }

    pub fn into_trits_allocated(&self, buf: &mut Trits<T1B1>) {
        buf.copy_raw_bytes(
            self.payload().to_inner(),
            PAYLOAD.trit_offset.start,
            PAYLOAD.trit_offset.length,
        );
        buf.copy_raw_bytes(
            self.address().to_inner(),
            ADDRESS.trit_offset.start,
            ADDRESS.trit_offset.length,
        );
        buf.copy_raw_bytes(
            self.obsolete_tag().to_inner(),
            OBSOLETE_TAG.trit_offset.start,
            OBSOLETE_TAG.trit_offset.length,
        );

        buf.copy_raw_bytes(
            self.bundle().to_inner(),
            BUNDLE.trit_offset.start,
            BUNDLE.trit_offset.length,
        );

        buf.copy_raw_bytes(
            self.branch().to_inner(),
            BRANCH.trit_offset.start,
            BRANCH.trit_offset.length,
        );

        buf.copy_raw_bytes(
            self.trunk().to_inner(),
            TRUNK.trit_offset.start,
            TRUNK.trit_offset.length,
        );

        buf.copy_raw_bytes(self.tag().to_inner(), TAG.trit_offset.start, TAG.trit_offset.length);

        buf.copy_raw_bytes(
            self.nonce().to_inner(),
            NONCE.trit_offset.start,
            NONCE.trit_offset.length,
        );

        let value_buf = TritBuf::<T1B1Buf>::try_from(*self.value().to_inner()).unwrap();

        buf.copy_raw_bytes(value_buf.as_slice(), VALUE.trit_offset.start, value_buf.len());

        let timestamp_buf = TritBuf::<T1B1Buf>::try_from(*self.timestamp().to_inner() as i64).unwrap();

        buf.copy_raw_bytes(
            timestamp_buf.as_slice(),
            TIMESTAMP.trit_offset.start,
            timestamp_buf.len(),
        );

        let attachment_ts_buf = TritBuf::<T1B1Buf>::try_from(*self.attachment_ts().to_inner() as i64).unwrap();

        buf.copy_raw_bytes(
            attachment_ts_buf.as_slice(),
            ATTACHMENT_TS.trit_offset.start,
            attachment_ts_buf.len(),
        );

        let attachment_lbts_buf = TritBuf::<T1B1Buf>::try_from(*self.timestamp().to_inner() as i64).unwrap();
        buf.copy_raw_bytes(
            attachment_lbts_buf.as_slice(),
            ATTACHMENT_LBTS.trit_offset.start,
            attachment_lbts_buf.len(),
        );

        let attachment_ubts_buf = TritBuf::<T1B1Buf>::try_from(*self.timestamp().to_inner() as i64).unwrap();
        buf.copy_raw_bytes(
            attachment_ubts_buf.as_slice(),
            ATTACHMENT_UBTS.trit_offset.start,
            attachment_ubts_buf.len(),
        );
    }

    pub fn payload(&self) -> &Payload {
        &self.payload
    }

    pub fn address(&self) -> &Address {
        &self.address
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn obsolete_tag(&self) -> &Tag {
        &self.obsolete_tag
    }

    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }

    pub fn index(&self) -> &Index {
        &self.index
    }

    pub fn last_index(&self) -> &Index {
        &self.last_index
    }

    pub fn bundle(&self) -> &Hash {
        &self.bundle
    }

    pub fn trunk(&self) -> &Hash {
        &self.trunk
    }

    pub fn branch(&self) -> &Hash {
        &self.branch
    }

    pub fn tag(&self) -> &Tag {
        &self.tag
    }

    pub fn attachment_ts(&self) -> &Timestamp {
        &self.attachment_ts
    }

    pub fn attachment_lbts(&self) -> &Timestamp {
        &self.attachment_lbts
    }

    pub fn attachment_ubts(&self) -> &Timestamp {
        &self.attachment_ubts
    }

    pub fn nonce(&self) -> &Nonce {
        &self.nonce
    }

    pub fn is_tail(&self) -> bool {
        self.index == Index(0)
    }

    pub fn is_head(&self) -> bool {
        self.index == self.last_index
    }

    pub const fn trits_len() -> usize {
        TRANSACTION_TRIT_LEN
    }
}

/// Transaction builder

#[derive(Debug)]
pub enum TransactionBuilderError {
    MissingField(&'static str),
    InvalidValue(i64),
}

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

    pub fn build(self) -> Result<Transaction, TransactionBuilderError> {
        let value = self
            .value
            .as_ref()
            .ok_or(TransactionBuilderError::MissingField("value"))?
            .0;

        if value.abs() > IOTA_SUPPLY {
            Err(TransactionBuilderError::InvalidValue(value))?;
        }

        Ok(Transaction {
            payload: self.payload.ok_or(TransactionBuilderError::MissingField("payload"))?,
            address: self.address.ok_or(TransactionBuilderError::MissingField("address"))?,
            value: Value(value),
            obsolete_tag: self
                .obsolete_tag
                .ok_or(TransactionBuilderError::MissingField("obsolete_tag"))?,
            timestamp: self
                .timestamp
                .ok_or(TransactionBuilderError::MissingField("timestamp"))?,
            index: self.index.ok_or(TransactionBuilderError::MissingField("index"))?,
            last_index: self
                .last_index
                .ok_or(TransactionBuilderError::MissingField("last_index"))?,
            tag: self.tag.ok_or(TransactionBuilderError::MissingField("tag"))?,
            bundle: self.bundle.ok_or(TransactionBuilderError::MissingField("bundle"))?,
            trunk: self.trunk.ok_or(TransactionBuilderError::MissingField("trunk"))?,
            branch: self.branch.ok_or(TransactionBuilderError::MissingField("branch"))?,
            attachment_ts: self
                .attachment_ts
                .ok_or(TransactionBuilderError::MissingField("attachment_ts"))?,
            attachment_lbts: self
                .attachment_lbts
                .ok_or(TransactionBuilderError::MissingField("attachment_lbts"))?,
            attachment_ubts: self
                .attachment_ubts
                .ok_or(TransactionBuilderError::MissingField("attachment_ubts"))?,
            nonce: self.nonce.ok_or(TransactionBuilderError::MissingField("nonce"))?,
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
