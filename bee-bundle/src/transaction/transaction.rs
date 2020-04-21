use crate::{
    constants::{
        ADDRESS,
        ATTACHMENT_LBTS,
        ATTACHMENT_TS,
        ATTACHMENT_UBTS,
        BRANCH,
        BUNDLE,
        INDEX,
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
        TransactionBuilder,
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
    InvalidNumericField(&'static str, num_conversions::TritsI64ConversionError),
    MissingField(&'static str),
    InvalidValue(i64),
    InvalidAddress,
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
    pub fn from_trits(buffer: &Trits<impl RawEncoding<Trit = Btrit> + ?Sized>) -> Result<Self, TransactionError> {
        let trits = buffer.encode::<T1B1Buf>();

        let transaction = TransactionBuilder::new()
            .with_payload(Payload(
                trits[PAYLOAD.trit_offset.start..PAYLOAD.trit_offset.start + PAYLOAD.trit_offset.length].to_buf(),
            ))
            .with_address(Address(
                trits[ADDRESS.trit_offset.start..ADDRESS.trit_offset.start + ADDRESS.trit_offset.length].to_buf(),
            ))
            .with_value(Value::from_inner_unchecked(
                i64::try_from(
                    trits[VALUE.trit_offset.start..VALUE.trit_offset.start + VALUE.trit_offset.length].to_buf(),
                )
                .map_err(|e| TransactionError::InvalidNumericField("value", e))?,
            ))
            .with_obsolete_tag(Tag(trits[OBSOLETE_TAG.trit_offset.start
                ..OBSOLETE_TAG.trit_offset.start + OBSOLETE_TAG.trit_offset.length]
                .to_buf()))
            .with_timestamp(Timestamp::from_inner_unchecked(
                i64::try_from(
                    trits[TIMESTAMP.trit_offset.start..TIMESTAMP.trit_offset.start + TIMESTAMP.trit_offset.length]
                        .to_buf(),
                )
                .map_err(|e| TransactionError::InvalidNumericField("timestamp", e))? as u64,
            ))
            .with_index(Index::from_inner_unchecked(
                i64::try_from(
                    trits[INDEX.trit_offset.start..INDEX.trit_offset.start + INDEX.trit_offset.length].to_buf(),
                )
                .map_err(|e| TransactionError::InvalidNumericField("index", e))? as usize,
            ))
            .with_last_index(Index::from_inner_unchecked(
                i64::try_from(
                    trits[LAST_INDEX.trit_offset.start..LAST_INDEX.trit_offset.start + LAST_INDEX.trit_offset.length]
                        .to_buf(),
                )
                .map_err(|e| TransactionError::InvalidNumericField("last_index", e))? as usize,
            ))
            .with_tag(Tag(trits
                [TAG.trit_offset.start..TAG.trit_offset.start + TAG.trit_offset.length]
                .to_buf()))
            .with_attachment_ts(Timestamp::from_inner_unchecked(
                i64::try_from(
                    trits[ATTACHMENT_TS.trit_offset.start
                        ..ATTACHMENT_TS.trit_offset.start + ATTACHMENT_TS.trit_offset.length]
                        .to_buf(),
                )
                .map_err(|e| TransactionError::InvalidNumericField("attachment_ts", e))? as u64,
            ))
            .with_bundle(Hash::from_inner_unchecked(
                trits[BUNDLE.trit_offset.start..BUNDLE.trit_offset.start + BUNDLE.trit_offset.length].to_buf(),
            ))
            .with_trunk(Hash::from_inner_unchecked(
                trits[TRUNK.trit_offset.start..TRUNK.trit_offset.start + TRUNK.trit_offset.length].to_buf(),
            ))
            .with_branch(Hash::from_inner_unchecked(
                trits[BRANCH.trit_offset.start..BRANCH.trit_offset.start + BRANCH.trit_offset.length].to_buf(),
            ))
            .with_attachment_lbts(Timestamp::from_inner_unchecked(
                i64::try_from(
                    trits[ATTACHMENT_LBTS.trit_offset.start
                        ..ATTACHMENT_LBTS.trit_offset.start + ATTACHMENT_LBTS.trit_offset.length]
                        .to_buf(),
                )
                .map_err(|e| TransactionError::InvalidNumericField("attachment_lbts", e))? as u64,
            ))
            .with_attachment_ubts(Timestamp::from_inner_unchecked(
                i64::try_from(
                    trits[ATTACHMENT_UBTS.trit_offset.start
                        ..ATTACHMENT_UBTS.trit_offset.start + ATTACHMENT_UBTS.trit_offset.length]
                        .to_buf(),
                )
                .map_err(|e| TransactionError::InvalidNumericField("attachment_ubts", e))? as u64,
            ))
            .with_nonce(Nonce(
                trits[NONCE.trit_offset.start..NONCE.trit_offset.start + NONCE.trit_offset.length].to_buf(),
            ))
            .build()?;

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

    // TODO rename ?
    // TODO return type ?
    pub fn get_timestamp(&self) -> u64 {
        match self.attachment_ts.to_inner() {
            0 => *self.timestamp.to_inner(),
            _ => *self.attachment_ts.to_inner() / 1000,
        }
    }

    pub const fn trit_len() -> usize {
        TRANSACTION_TRIT_LEN
    }
}

#[derive(Default)]
pub struct Transactions(pub(crate) Vec<Transaction>);

impl Transactions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, index: usize) -> Option<&Transaction> {
        self.0.get(index)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push(&mut self, transaction: Transaction) {
        self.0.push(transaction);
    }
}
