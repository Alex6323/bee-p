mod trit;

pub use trit::Trit;

use std::ops::Range;

pub trait RawEncoding {
    fn len(&self) -> usize;
    unsafe fn get_unchecked(&self, index: usize) -> Trit;
    unsafe fn set_unchecked(&mut self, index: usize, trit: Trit);
    unsafe fn slice_unchecked(&self, range: Range<usize>) -> &Self;
    unsafe fn slice_unchecked_mut(&mut self, range: Range<usize>) -> &mut Self;
}

pub trait RawEncodingBuf {
    type Slice: RawEncoding + ?Sized;

    fn new() -> Self
    where
        Self: Sized;

    fn push(&mut self, trit: Trit);

    fn from_trits<T: Into<Trit> + Clone>(trits: &[T]) -> Self
    where
        Self: Sized,
    {
        let mut this = Self::new();
        for trit in trits {
            this.push(trit.clone().into());
        }
        this
    }

    fn as_slice(&self) -> &Self::Slice;
    fn as_slice_mut(&mut self) -> &mut Self::Slice;

    fn into_encoding<T: RawEncodingBuf>(this: TritBuf<Self>) -> TritBuf<T>
    where
        Self: Sized,
    {
        this.iter().collect()
    }
}
