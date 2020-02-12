use bee_ternary::{
    Trits,
    TritBuf,
    raw::{RawEncoding, RawEncodingBuf},
};

pub(crate) trait Sealed {}

impl<'a, T: RawEncoding> Sealed for &'a Trits<T> {}
impl<T: RawEncodingBuf> Sealed for TritBuf<T> {}
