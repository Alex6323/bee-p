pub mod trit;
pub mod tryte;
pub mod raw;
pub mod t1b1;
pub mod t4b1;
pub mod util;

use std::{
    ops::{Deref, DerefMut, Range, Index, IndexMut},
    cmp::PartialEq,
    iter::FromIterator,
    any,
    fmt,
};
use crate::raw::{RawEncoding, RawEncodingBuf};

// Reexports
pub use crate::{
    tryte::{Tryte, IsTryte, TRYTE_ALPHABET},
    trit::Trit,
    t1b1::{T1B1, T1B1Buf},
    t4b1::{T4B1, T4B1Buf},
};

// ONLY TEMPORARY
// re-export iota-conversion
pub use iota_conversion;

#[repr(transparent)]
pub struct Trits<T: RawEncoding + ?Sized = T1B1>(T);

impl<T: RawEncoding + ?Sized> Trits<T> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub unsafe fn get_unchecked(&self, index: usize) -> Trit {
        self.0.get_unchecked(index).into()
    }

    pub unsafe fn set_unchecked(&mut self, index: usize, trit: Trit) {
        self.0.set_unchecked(index, trit.into());
    }

    pub fn get(&self, index: usize) -> Option<Trit> {
        if index < self.0.len() {
            unsafe { Some(self.get_unchecked(index)) }
        } else {
            None
        }
    }

    pub fn set(&mut self, index: usize, trit: Trit) {
        if index < self.0.len() {
            unsafe { self.set_unchecked(index, trit) };
        }
    }

    pub fn iter(&self) -> impl Iterator<Item=Trit> + '_ {
        (0..self.0.len()).map(move |idx| unsafe { self.0.get_unchecked(idx).into() })
    }

    pub fn slice(&self, range: Range<usize>) -> &Self {
        assert!(range.end >= range.start && range.end <= self.len());
        unsafe { &*(self.0.slice_unchecked(range) as *const _ as *const Self) }
    }

    pub fn slice_mut(&mut self, range: Range<usize>) -> &mut Self {
        assert!(range.end >= range.start && range.end <= self.len());
        unsafe { &mut *(self.0.slice_unchecked_mut(range) as *mut _ as *mut Self) }
    }

    pub fn copy_from<U: RawEncoding + ?Sized>(&mut self, trits: &Trits<U>) {
        assert!(self.len() == trits.len());
        for (i, trit) in trits.iter().enumerate() {
            unsafe { self.set_unchecked(i, trit); }
        }
    }

    pub fn fill(&mut self, trit: Trit) {
        for i in 0..self.len() {
            self.set(i, trit);
        }
    }

    pub fn chunks(&self, chunk_len: usize) -> impl Iterator<Item=&Self> + '_ {
        (0..self.len())
            .step_by(chunk_len)
            .map(move |i| self.slice(i..(i + chunk_len).min(self.len())))
    }

    pub fn chunks_mut(&mut self, chunk_len: usize) -> impl Iterator<Item=&mut Self> + '_ {
        (0..self.len())
            .step_by(chunk_len)
            .scan(self, move |this, index| {
                let idx = chunk_len.min(this.len());
                let (a, b) = Trits::split_at_mut(this, idx);
                *this = b;
                Some(a)
            })
    }

    // Helper
    // TODO: Make this public? Is it needed?
    fn split_at_mut<'a>(this: &mut &'a mut Self, idx: usize) -> (&'a mut Self, &'a mut Self) {
        assert!(idx < this.len());
        (
            unsafe { &mut *(this.0.slice_unchecked_mut(0..idx) as *mut _ as *mut Self) },
            unsafe { &mut *(this.0.slice_unchecked_mut(idx..this.len()) as *mut _ as *mut Self) },
        )
    }
}

impl<T: RawEncoding + ?Sized, U: RawEncoding + ?Sized> PartialEq<Trits<U>> for Trits<T> {
    fn eq(&self, other: &Trits<U>) -> bool {
        self.len() == other.len() && self
            .iter()
            .zip(other.iter())
            .all(|(a, b)| a == b)
    }
}

impl<'a, T: RawEncoding> fmt::Debug for &'a Trits<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Trits<{}> [", any::type_name::<T>())?;
        for (i, trit) in self.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:?}", trit)?;
        }
        write!(f, "]")
    }
}

impl<T: RawEncoding> Index<Range<usize>> for Trits<T> {
    type Output = Self;

    fn index(&self, range: Range<usize>) -> &Self::Output {
        self.slice(range)
    }
}

impl<T: RawEncoding> IndexMut<Range<usize>> for Trits<T> {
    fn index_mut(&mut self, range: Range<usize>) -> &mut Self::Output {
        self.slice_mut(range)
    }
}

#[repr(transparent)]
pub struct TritBuf<T: RawEncodingBuf = T1B1Buf>(T);

impl<T: RawEncodingBuf> TritBuf<T> {
    pub fn new() -> Self {
        Self(T::new())
    }

    // TODO: Make public when original purged
    fn with_capacity(cap: usize) -> Self {
        // TODO: Allocate capacity
        Self::new()
    }

    pub fn filled(len: usize, trit: Trit) -> Self {
        let mut this = Self::with_capacity(len);
        for _ in 0..len {
            this.push(trit);
        }
        this
    }

    pub fn zeros(len: usize) -> Self {
        Self::filled(len, Trit::Zero)
    }

    pub fn from_trits<U: Into<Trit> + Clone>(trits: &[U]) -> Self {
        Self(T::from_trits(trits))
    }

    // TODO: Is this a good API feature?
    pub fn from_i8_unchecked(trits: &[i8]) -> Self {
        // TODO: Don't check
        Self::from_trits(trits)
    }

    pub fn push(&mut self, trit: Trit) {
        self.0.push(trit.into());
    }

    pub fn as_slice(&self) -> &Trits<T::Slice> {
        unsafe { &*(self.0.as_slice() as *const T::Slice as *const Trits<T::Slice>) }
    }

    pub fn as_slice_mut(&mut self) -> &mut Trits<T::Slice> {
        unsafe { &mut *(self.0.as_slice_mut() as *mut T::Slice as *mut Trits<T::Slice>) }
    }

    pub fn into_encoding<U: RawEncodingBuf>(self) -> TritBuf<U> {
        T::into_encoding(self)
    }
}

impl<T: RawEncodingBuf, U: RawEncodingBuf> PartialEq<TritBuf<U>> for TritBuf<T>
    where
        T::Slice: RawEncoding,
        U::Slice: RawEncoding,
{
    fn eq(&self, other: &TritBuf<U>) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: RawEncodingBuf> Deref for TritBuf<T> {
    type Target = Trits<T::Slice>;

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T: RawEncodingBuf> DerefMut for TritBuf<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}

impl<T: RawEncodingBuf> FromIterator<Trit> for TritBuf<T> {
    fn from_iter<I: IntoIterator<Item=Trit>>(iter: I) -> Self {
        let mut this = Self::new();

        for trit in iter {
            this.push(trit);
        }

        this
    }
}

impl<T: RawEncodingBuf> Index<Range<usize>> for TritBuf<T> {
    type Output = Trits<T::Slice>;

    fn index(&self, range: Range<usize>) -> &Self::Output {
        self.slice(range)
    }
}

impl<T: RawEncodingBuf> IndexMut<Range<usize>> for TritBuf<T> {
    fn index_mut(&mut self, range: Range<usize>) -> &mut Self::Output {
        self.slice_mut(range)
    }
}

impl<T: RawEncodingBuf> fmt::Debug for TritBuf<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TritBuf<{}> [", any::type_name::<T>())?;
        for (i, trit) in self.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:?}", trit)?;
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare() {
        fn slices_eq(a: &Trits<T4B1>, b: &Trits<T4B1>) -> bool {
            a
                .iter()
                .zip(b.iter())
                .all(|(a, b)| a == b)
        }

        let mut a = TritBuf::<T4B1Buf>::from_trits(&[1i8, -1, 0, 1, 0])
            .into_encoding::<T1B1Buf>()
            .into_encoding::<T4B1Buf>();

        a.set(2, Trit::MinusOne);

        let b = TritBuf::<T4B1Buf>::from_trits(&[-1i8, -1, 1]);

        assert!(slices_eq(&a[1..5], &b));
    }
}
