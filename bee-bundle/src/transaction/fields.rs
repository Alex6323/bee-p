use crate::constants::{
    ADDRESS,
    ADDRESS_TRIT_LEN,
    HASH_TRIT_LEN,
    NONCE,
    NONCE_TRIT_LEN,
    PAYLOAD,
    PAYLOAD_TRIT_LEN,
    TAG,
    TAG_TRIT_LEN,
};

use bee_ternary::{
    T1B1Buf,
    TritBuf,
    Trits,
    T1B1,
};

use std::{
    cmp::PartialEq,
    fmt,
    hash,
};

#[derive(Debug)]
pub enum TransactionFieldError {
    FieldWrongLength,
    FieldDeserializationError,
}

pub trait TransactionField: Sized + TransactionFieldType {
    type Inner: ToOwned + ?Sized;
    fn try_from_inner(buffer: <Self::Inner as ToOwned>::Owned) -> Result<Self, TransactionFieldError>;
    fn from_inner_unchecked(buffer: <Self::Inner as ToOwned>::Owned) -> Self;

    fn to_inner(&self) -> &Self::Inner;

    fn trit_len() -> usize;
}

pub trait NumTritsOfValue {
    fn num_trits(&self) -> usize;
}

pub trait TransactionFieldType {
    type InnerType: NumTritsOfValue;

    fn is_trits_type() -> bool;
}

impl NumTritsOfValue for TritBuf<T1B1Buf> {
    fn num_trits(&self) -> usize {
        self.len()
    }
}

impl NumTritsOfValue for i64 {
    fn num_trits(&self) -> usize {
        unimplemented!();
    }
}

impl NumTritsOfValue for u64 {
    fn num_trits(&self) -> usize {
        unimplemented!();
    }
}

impl NumTritsOfValue for usize {
    fn num_trits(&self) -> usize {
        unimplemented!();
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Payload(pub(crate) TritBuf<T1B1Buf>);

impl Payload {
    pub fn zeros() -> Self {
        Self(TritBuf::zeros(PAYLOAD.trit_offset.length))
    }

    pub fn trit_len() -> usize {
        PAYLOAD_TRIT_LEN
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Address(pub(crate) TritBuf<T1B1Buf>);

impl Address {
    pub fn zeros() -> Self {
        Self(TritBuf::zeros(ADDRESS.trit_offset.length))
    }

    pub fn trit_len() -> usize {
        ADDRESS_TRIT_LEN
    }
}

impl Eq for Address {}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Value(pub(crate) i64);

impl Value {
    pub fn trit_len() -> usize {
        unimplemented!();
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tag(pub(crate) TritBuf<T1B1Buf>);

impl Tag {
    pub fn zeros() -> Self {
        Self(TritBuf::zeros(TAG.trit_offset.length))
    }

    pub fn trit_len() -> usize {
        TAG_TRIT_LEN
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Timestamp(pub(crate) u64);

impl Timestamp {
    pub fn trit_len() -> usize {
        unimplemented!();
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Index(pub(crate) usize);

impl Index {
    pub fn trit_len() -> usize {
        unimplemented!();
    }
}

#[derive(Copy, Clone)]
// TODO pub ?
pub struct Hash(pub [i8; 243]);

impl Hash {
    pub fn zeros() -> Self {
        Self([0; 243])
    }

    pub fn as_bytes(&self) -> &[i8] {
        &self.0
    }

    pub fn as_trits(&self) -> &Trits<T1B1> {
        unsafe { Trits::from_raw_unchecked(self.as_bytes(), 243) }
    }

    pub fn weight(&self) -> u8 {
        let mut weight = 0u8;

        for i in (0..self.0.len()).rev() {
            if self.0[i] != 0 {
                break;
            }
            weight = weight + 1;
        }

        weight
    }

    pub fn trit_len() -> usize {
        HASH_TRIT_LEN
    }
}

impl PartialEq for Hash {
    fn eq(&self, other: &Self) -> bool {
        self.0.iter().zip(other.0.iter()).all(|(a, b)| a == b)
    }
}
impl Eq for Hash {}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.as_trits())
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.as_trits())
    }
}

impl hash::Hash for Hash {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        self.0.hash(hasher)
    }
}

impl TransactionFieldType for Hash {
    type InnerType = TritBuf<T1B1Buf>;

    fn is_trits_type() -> bool {
        true
    }
}

impl TransactionField for Hash {
    // TODO why Trits and not TritBuf ?
    type Inner = Trits<T1B1>;

    fn to_inner(&self) -> &Self::Inner {
        self.as_trits()
    }

    fn trit_len() -> usize {
        243
    }

    fn try_from_inner(buf: <Self::Inner as ToOwned>::Owned) -> Result<Self, TransactionFieldError> {
        if buf.len() != Self::trit_len() {
            Err(TransactionFieldError::FieldWrongLength)?
        }

        Ok(Self::from_inner_unchecked(buf))
    }

    fn from_inner_unchecked(buf: <Self::Inner as ToOwned>::Owned) -> Self {
        let mut trits = [0; 243];
        trits.copy_from_slice(buf.as_i8_slice());

        Self(trits)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Nonce(pub(crate) TritBuf<T1B1Buf>);

impl Nonce {
    pub fn zeros() -> Self {
        Self(TritBuf::zeros(NONCE.trit_offset.length))
    }

    pub fn trit_len() -> usize {
        NONCE_TRIT_LEN
    }
}

macro_rules! impl_transaction_field {
    ( $($field_name:ident),+ $(,)?) => {
        $(
            impl TransactionField for $field_name {

                type Inner = <$field_name as TransactionFieldType>::InnerType;

                fn to_inner(&self) -> &Self::Inner {
                    &self.0
                }

                fn try_from_inner(val: Self::Inner) -> Result<Self, TransactionFieldError> {
                    if $field_name::is_trits_type() && val.num_trits() != $field_name::trit_len() {
                        Err(TransactionFieldError::FieldWrongLength)?
                    }
                    Ok(Self::from_inner_unchecked(val))
                }

                fn from_inner_unchecked(val: Self::Inner) -> Self {
                    Self(val)
                }

                fn trit_len() -> usize {
                   Self::trit_len()
                }
            }
        )+
    }
}

macro_rules! impl_transaction_field_type_for_tritbuf_fields {
    ( $($field_name:ident),+ $(,)?) => {
        $(
            impl TransactionFieldType for $field_name {
                type InnerType = TritBuf<T1B1Buf>;
                fn is_trits_type() -> bool {true}
            }
        )+
    }
}

impl TransactionFieldType for Value {
    type InnerType = i64;

    fn is_trits_type() -> bool {
        false
    }
}

impl TransactionFieldType for Index {
    type InnerType = usize;

    fn is_trits_type() -> bool {
        false
    }
}

impl TransactionFieldType for Timestamp {
    type InnerType = u64;

    fn is_trits_type() -> bool {
        false
    }
}

macro_rules! impl_hash_trait {
    ( $($field_name:ident),+ $(,)?) => {
        $(
            impl hash::Hash for $field_name {
                fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
                       self.0.hash(hasher)
                }
            }
        )+
    }
}

impl_transaction_field_type_for_tritbuf_fields!(Payload, Address, Tag, Nonce);
impl_transaction_field!(Payload, Address, Tag, Nonce, Index, Value, Timestamp);
impl_hash_trait!(Address);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_weigth() {
        for i in 0..20 {
            let mut trits = [0i8; 243];
            trits[243 - i - 1] = 1;
            unsafe {
                let hash = Hash::try_from_inner(Trits::<T1B1>::from_raw_unchecked(&trits, 243).to_buf()).unwrap();
                assert_eq!(hash.weight(), i as u8);
            }
        }
    }
}
