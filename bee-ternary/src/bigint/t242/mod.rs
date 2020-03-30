//! This module contains logic to convert an integer encoded by 243 trits to the same
//! integer encoded by 384 bits (or 48 signed bytes, `i8`).
//!
//! At the core of this a slice of binary-coded, balanced trits is interpreted
//! fanned-out `t243`, where `t243` is used analogous to `i64` or `u64`. If the latter
//! are 64-bit signed/unsigned integer types, then `t243` is a 243-trit integer type.
//! Analogous to fanning out a `u64` into 64 individual bits, `t243` is fanned out into
//! 243 trits, each (rather inefficiently) represented by one `u8`.

use std::{
    cmp::Ordering,
    convert::{
        TryFrom,
        TryInto,
    },
};

use crate::{
    raw::RawEncoding,
    ShiftTernary,
    Trit,
    Trits,
    TritBuf,
    T1B1Buf,
    Btrit,
    Utrit,
    bigint::{
        u384,
        common::{
            BigEndian,
            Error,
            LittleEndian,
            U8Repr,
            U32Repr,
        },
        I384,
        T243,
        U384,
    },
};

mod constants;

def_and_impl_ternary!(T242, 242);

impl<T: Trit> T242<T> {
    pub fn into_t243(self) -> T243<T> {
        let mut trit_buf = self.into_inner();
        trit_buf.push(T::zero());
        T243::from_trit_buf(trit_buf)
    }
}

impl T242<Btrit> {
    pub fn from_i384_be_u8repr_ignoring_msd(value: I384<BigEndian, U8Repr>) -> Self {
        let value: I384<LittleEndian, U8Repr> = value.into();
        let value: I384<LittleEndian, U32Repr> = value.into();
        let mut unsigned_binary = value.as_u384();
        unsigned_binary.add_inplace(*u384::LE_U32_HALF_MAX_T242);
        let t243_utrit: T243<Utrit> = unsigned_binary.into();
        let t243_btrit = t243_utrit.into_shifted();
        t243_btrit.into_t242()
    }
    
    fn try_from_i384(value: I384<LittleEndian, U32Repr>) -> Result<Self, Error> {
        let mut unsigned_binary = value.as_u384();
        unsigned_binary.add_inplace(*u384::LE_U32_HALF_MAX_T242);
        if unsigned_binary > *u384::LE_U32_MAX_T242 {
            Err(Error::BinaryExceedsTernaryRange)?
        }
        let unsigned_ternary: T243<Utrit> = unsigned_binary.into();
        let signed_ternary = unsigned_ternary.into_shifted();
        Ok(signed_ternary.into_t242())
    }
}

impl TryFrom<I384<BigEndian, U8Repr>> for T242<Btrit> {
    type Error = Error;

    fn try_from(value: I384<BigEndian, U8Repr>) -> Result<Self, Self::Error> {
        let as_littleendian: I384<LittleEndian, U8Repr> = value.into();
        let as_littleendian_u32repr: I384<LittleEndian, U32Repr> = as_littleendian.into();
        as_littleendian_u32repr.try_into()
    }
}

impl TryFrom<I384<BigEndian, U32Repr>> for T242<Btrit> {
    type Error = Error;

    fn try_from(value: I384<BigEndian, U32Repr>) -> Result<Self, Self::Error> {
        let as_littleendian: I384<LittleEndian, U32Repr> = value.into();
        as_littleendian.try_into()
    }
}

impl TryFrom<I384<LittleEndian, U8Repr>> for T242<Btrit> {
    type Error = Error;

    fn try_from(value: I384<LittleEndian, U8Repr>) -> Result<Self, Self::Error> {
        let as_u32repr: I384<LittleEndian, U32Repr> = value.into();
        as_u32repr.try_into()
    }
}

impl TryFrom<I384<LittleEndian, U32Repr>> for T242<Btrit> {
    type Error = Error;

    fn try_from(value: I384<LittleEndian, U32Repr>) -> Result<Self, Self::Error> {
        Self::try_from_i384(value)
    }
}

impl From<U384<LittleEndian, U32Repr>> for T242<Utrit> {
    fn from(binary_value: U384<LittleEndian, U32Repr>) -> Self {
        let ternary_value: T243<Utrit> = binary_value.into();
        ternary_value.into_t242()
    }
}
