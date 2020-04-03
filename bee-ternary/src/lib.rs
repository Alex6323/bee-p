use std::{
    convert::TryFrom,
    slice,
};

pub mod bigint;
pub mod raw;
pub mod t1b1;
pub mod t2b1;
pub mod t3b1;
pub mod t4b1;
pub mod t5b1;
pub mod trit;
pub mod tryte;

#[cfg(feature = "serde1")]
mod serde;

use crate::raw::{
    RawEncoding,
    RawEncodingBuf,
};
use std::{
    any,
    cmp::{
        self,
        Ordering,
    },
    fmt,
    hash,
    iter::FromIterator,
    ops::{
        Deref,
        DerefMut,
        Index,
        IndexMut,
        Range,
    },
    borrow::{Borrow, BorrowMut},
};

// Reexports
pub use crate::{
    t1b1::{
        T1B1Buf,
        T1B1,
    },
    t2b1::{
        T2B1Buf,
        T2B1,
    },
    t3b1::{
        T3B1Buf,
        T3B1,
    },
    t4b1::{
        T4B1Buf,
        T4B1,
    },
    t5b1::{
        T5B1Buf,
        T5B1,
    },
    trit::{
        Btrit,
        ShiftTernary,
        Trit,
        Utrit,
    },
    tryte::{
        Tryte,
        TryteBuf,
    },
};

#[derive(Debug)]
pub enum Error {
    InvalidRepr,
}

#[derive(Hash)]
#[repr(transparent)]
pub struct Trits<T: RawEncoding + ?Sized = T1B1<Btrit>>(T);

impl<T> Trits<T>
where
    T: RawEncoding + ?Sized,
{
    pub fn empty() -> &'static Self {
        unsafe { &*(T::empty() as *const _ as *const Self) }
    }

    pub unsafe fn from_raw_unchecked(raw: &[i8], num_trits: usize) -> &Self {
        &*(T::from_raw_unchecked(raw, num_trits) as *const _ as *const _)
    }

    pub unsafe fn from_raw_unchecked_mut(raw: &mut [i8], num_trits: usize) -> &mut Self {
        &mut *(T::from_raw_unchecked(raw, num_trits) as *const _ as *mut _)
    }

    pub fn try_from_raw(raw: &[i8], num_trits: usize) -> Result<&Self, Error> {
        if raw.iter().all(T::is_valid) {
            Ok(unsafe { Self::from_raw_unchecked(raw, num_trits) })
        } else {
            Err(Error::InvalidRepr)
        }
    }

    pub fn try_from_raw_mut(raw: &mut [i8], num_trits: usize) -> Result<&mut Self, Error> {
        if raw.iter().all(T::is_valid) {
            Ok(unsafe { Self::from_raw_unchecked_mut(raw, num_trits) })
        } else {
            Err(Error::InvalidRepr)
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn as_i8_slice(&self) -> &[i8] {
        self.0.as_i8_slice()
    }

    pub unsafe fn as_i8_slice_mut(&mut self) -> &mut [i8] {
        self.0.as_i8_slice_mut()
    }

    pub unsafe fn get_unchecked(&self, index: usize) -> T::Trit {
        self.0.get_unchecked(index)
    }

    pub unsafe fn set_unchecked(&mut self, index: usize, trit: T::Trit) {
        self.0.set_unchecked(index, trit);
    }

    pub fn get(&self, index: usize) -> Option<T::Trit> {
        if index < self.0.len() {
            unsafe { Some(self.get_unchecked(index)) }
        } else {
            None
        }
    }

    pub fn set(&mut self, index: usize, trit: T::Trit) {
        if index < self.0.len() {
            unsafe { self.set_unchecked(index, trit) };
        } else {
            panic!(
                "Attempt to set trit at index {}, but length of slice is {}",
                index,
                self.len()
            );
        }
    }

    pub fn trits(&self) -> impl DoubleEndedIterator<Item = T::Trit> + ExactSizeIterator<Item = T::Trit> + '_ {
        (0..self.0.len()).map(move |idx| unsafe { self.0.get_unchecked(idx) })
    }

    pub fn slice(&self, range: Range<usize>) -> &Self {
        assert!(range.end >= range.start && range.end <= self.len());
        unsafe { &*(self.0.slice_unchecked(range) as *const _ as *const Self) }
    }

    pub fn slice_mut(&mut self, range: Range<usize>) -> &mut Self {
        assert!(range.end >= range.start && range.end <= self.len());
        unsafe { &mut *(self.0.slice_unchecked_mut(range) as *mut _ as *mut Self) }
    }

    pub fn copy_from<U: RawEncoding<Trit = T::Trit> + ?Sized>(&mut self, trits: &Trits<U>) {
        assert!(self.len() == trits.len());
        for (i, trit) in trits.trits().enumerate() {
            unsafe {
                self.set_unchecked(i, trit);
            }
        }
    }

    pub fn fill(&mut self, trit: T::Trit) {
        for i in 0..self.len() {
            self.set(i, trit);
        }
    }

    pub fn to_buf<U>(&self) -> TritBuf<U>
    where
        U: RawEncodingBuf,
        U::Slice: RawEncoding<Trit = T::Trit>,
    {
        self.trits().collect()
    }

    pub fn chunks(
        &self,
        chunk_len: usize,
    ) -> impl DoubleEndedIterator<Item = &Self> + ExactSizeIterator<Item = &Self> + '_ {
        assert!(chunk_len > 0);
        (0..self.len())
            .step_by(chunk_len)
            .map(move |i| self.slice(i..(i + chunk_len).min(self.len())))
    }

    pub fn encode<U>(&self) -> TritBuf<U>
    where
        U: RawEncodingBuf,
        U::Slice: RawEncoding<Trit = T::Trit>,
    {
        self.trits().collect()
    }
}

impl<T: Trit> Trits<T1B1<T>> {
    pub fn as_raw_slice(&self) -> &[T] {
        self.0.as_raw_slice()
    }

    pub fn as_raw_slice_mut(&mut self) -> &mut [T] {
        self.0.as_raw_slice_mut()
    }

    // Q: Why isn't this method on Trits<T>?
    // A: Because overlapping slice lifetimes make this unsound on squashed encodings
    pub fn chunks_mut(&mut self, chunk_len: usize) -> impl Iterator<Item = &mut Self> + '_ {
        assert!(chunk_len > 0);
        (0..self.len()).step_by(chunk_len).scan(self, move |this, _| {
            let idx = chunk_len.min(this.len());
            let (a, b) = Trits::split_at_mut(this, idx);
            *this = b;
            Some(a)
        })
    }

    // Helper
    // TODO: Make this public? Is it needed?
    // Q: Why isn't this method on Trits<T>?
    // A: Because overlapping slice lifetimes make this unsound on squashed encodings
    fn split_at_mut<'a>(this: &mut &'a mut Self, idx: usize) -> (&'a mut Self, &'a mut Self) {
        assert!(idx <= this.len());
        (
            unsafe { &mut *(this.0.slice_unchecked_mut(0..idx) as *mut _ as *mut Self) },
            unsafe { &mut *(this.0.slice_unchecked_mut(idx..this.len()) as *mut _ as *mut Self) },
        )
    }

    pub fn iter<'a>(&'a self) -> slice::Iter<'a, T> {
        self.as_raw_slice().iter()
    }

    pub fn iter_mut<'a>(&'a mut self) -> slice::IterMut<'a, T> {
        self.as_raw_slice_mut().iter_mut()
    }
}

impl<T, U> cmp::PartialEq<Trits<U>> for Trits<T>
where
    T: RawEncoding + ?Sized,
    U: RawEncoding<Trit = T::Trit> + ?Sized,
{
    fn eq(&self, other: &Trits<U>) -> bool {
        self.len() == other.len() && self.trits().zip(other.trits()).all(|(a, b)| a == b)
    }
}

impl<T, U> cmp::PartialOrd<Trits<U>> for Trits<T>
where
    T: RawEncoding + ?Sized,
    U: RawEncoding<Trit = T::Trit> + ?Sized,
    T::Trit: cmp::PartialOrd,
{
    fn partial_cmp(&self, other: &Trits<U>) -> Option<Ordering> {
        if self.len() != other.len() {
            return None;
        }

        for (a, b) in self.trits().zip(other.trits()) {
            match a.partial_cmp(&b) {
                Some(Ordering::Equal) => continue,
                other_order => return other_order,
            }
        }

        Some(Ordering::Equal)
    }
}

impl<'a, T: RawEncoding + ?Sized> fmt::Debug for &'a Trits<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Trits<{}> [", any::type_name::<T>())?;
        for (i, trit) in self.trits().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:?}", trit)?;
        }
        write!(f, "]")
    }
}

impl<T: RawEncoding + ?Sized> Index<Range<usize>> for Trits<T> {
    type Output = Self;

    fn index(&self, range: Range<usize>) -> &Self::Output {
        self.slice(range)
    }
}

impl<T: RawEncoding + ?Sized> IndexMut<Range<usize>> for Trits<T> {
    fn index_mut(&mut self, range: Range<usize>) -> &mut Self::Output {
        self.slice_mut(range)
    }
}

impl<T: RawEncoding + ?Sized> ToOwned for Trits<T> {
    type Owned = TritBuf<T::Buf>;

    fn to_owned(&self) -> Self::Owned {
        self.to_buf()
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub struct TritBuf<T: RawEncodingBuf = T1B1Buf<Btrit>>(T);

impl<T: RawEncodingBuf> TritBuf<T> {
    pub fn new() -> Self {
        Self(T::new())
    }

    // TODO: Make public when original purged
    fn with_capacity(_cap: usize) -> Self {
        // TODO: Allocate capacity
        Self::new()
    }

    pub fn filled(len: usize, trit: <T::Slice as RawEncoding>::Trit) -> Self {
        let mut this = Self::with_capacity(len);
        for _ in 0..len {
            this.push(trit);
        }
        this
    }

    pub fn zeros(len: usize) -> Self {
        Self::filled(len, <T::Slice as RawEncoding>::Trit::zero())
    }

    pub fn from_trits(trits: &[<T::Slice as RawEncoding>::Trit]) -> Self {
        Self(T::from_trits(trits))
    }

    // TODO: Is this a good API feature? No, it's not. Kill it with fire.
    #[deprecated]
    pub fn from_i8_unchecked(trits: &[i8]) -> Self {
        trits
            .iter()
            .map(|t| <T::Slice as RawEncoding>::Trit::try_from(*t))
            .collect::<Result<Self, _>>()
            .unwrap_or_else(|_| panic!("Invalid i8 when converting to trit."))
    }

    pub fn push(&mut self, trit: <T::Slice as RawEncoding>::Trit) {
        self.0.push(trit);
    }

    pub fn pop(&mut self) -> Option<<T::Slice as RawEncoding>::Trit> {
        self.0.pop()
    }

    pub fn as_slice(&self) -> &Trits<T::Slice> {
        unsafe { &*(self.0.as_slice() as *const T::Slice as *const Trits<T::Slice>) }
    }

    pub fn as_slice_mut(&mut self) -> &mut Trits<T::Slice> {
        unsafe { &mut *(self.0.as_slice_mut() as *mut T::Slice as *mut Trits<T::Slice>) }
    }
}

impl<T> TritBuf<T1B1Buf<T>>
where
    T: Trit,
    T::Target: Trit,
{
    pub fn into_shifted(self) -> TritBuf<T1B1Buf<<T as ShiftTernary>::Target>> {
        TritBuf(self.0.into_shifted())
    }
}

impl<T: RawEncodingBuf, U: RawEncodingBuf> PartialEq<TritBuf<U>> for TritBuf<T>
where
    T::Slice: RawEncoding,
    U::Slice: RawEncoding<Trit = <T::Slice as RawEncoding>::Trit>,
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

impl<T: RawEncodingBuf> FromIterator<<T::Slice as RawEncoding>::Trit> for TritBuf<T> {
    fn from_iter<I: IntoIterator<Item = <T::Slice as RawEncoding>::Trit>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut this = Self::with_capacity(iter.size_hint().0);
        for trit in iter {
            this.push(trit);
        }
        this
    }
}

impl<T> hash::Hash for TritBuf<T>
where
    T: RawEncodingBuf,
    T::Slice: hash::Hash,
{
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        (**self).hash(hasher)
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
        for (i, trit) in self.trits().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:?}", trit)?;
        }
        write!(f, "]")
    }
}

impl<T: RawEncodingBuf> Borrow<Trits<T::Slice>> for TritBuf<T> {
    fn borrow(&self) -> &Trits<T::Slice> { self.as_slice() }
}

impl<T: RawEncodingBuf> BorrowMut<Trits<T::Slice>> for TritBuf<T> {
    fn borrow_mut(&mut self) -> &mut Trits<T::Slice> { self.as_slice_mut() }
}
