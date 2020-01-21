pub mod trit;
pub mod tryte;
pub mod raw;
pub mod t1b1;
pub mod t4b1;

use std::{
    ops::{Deref, DerefMut, Range, Index, IndexMut},
    iter::FromIterator,
    any,
    fmt,
};
use crate::raw::{RawEncoding, RawEncodingBuf};

// Reexports
pub use crate::{
    tryte::{Tryte, IsTryte},
    trit::Trit,
    t1b1::{T1B1, T1B1Buf},
    t4b1::{T4B1, T4B1Buf},
};

// ONLY TEMPORARY
// re-export iota-conversion
pub use iota_conversion;

#[repr(transparent)]
pub struct TritSlice<T: RawEncoding + ?Sized = T1B1>(T);

impl<T: RawEncoding + ?Sized> TritSlice<T> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, index: usize) -> Option<Trit> {
        if index < self.0.len() {
            unsafe { Some(self.0.get_unchecked(index).into()) }
        } else {
            None
        }
    }

    pub fn set(&mut self, index: usize, trit: Trit) {
        if index < self.0.len() {
            unsafe { self.0.set_unchecked(index, trit.into()) };
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
}

impl<'a, T: RawEncoding> fmt::Debug for &'a TritSlice<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TritSlice<{}> [", any::type_name::<T>())?;
        for (i, trit) in self.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:?}", trit)?;
        }
        write!(f, "]")
    }
}

impl<T: RawEncoding> Index<Range<usize>> for TritSlice<T> {
    type Output = Self;

    fn index(&self, range: Range<usize>) -> &Self::Output {
        self.slice(range)
    }
}

impl<T: RawEncoding> IndexMut<Range<usize>> for TritSlice<T> {
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

    pub fn from_trits<U: Into<Trit> + Clone>(trits: &[U]) -> Self {
        Self(T::from_trits(trits))
    }

    pub fn push(&mut self, trit: Trit) {
        self.0.push(trit.into());
    }

    pub fn as_slice(&self) -> &TritSlice<T::Slice> {
        unsafe { &*(self.0.as_slice() as *const T::Slice as *const TritSlice<T::Slice>) }
    }

    pub fn as_slice_mut(&mut self) -> &mut TritSlice<T::Slice> {
        unsafe { &mut *(self.0.as_slice_mut() as *mut T::Slice as *mut TritSlice<T::Slice>) }
    }

    pub fn into_encoding<U: RawEncodingBuf>(self) -> TritBuf<U> {
        T::into_encoding(self)
    }
}

impl<T: RawEncodingBuf> Deref for TritBuf<T> {
    type Target = TritSlice<T::Slice>;

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
    type Output = TritSlice<T::Slice>;

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
        fn slices_eq(a: &TritSlice<T4B1>, b: &TritSlice<T4B1>) -> bool {
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
