use std::{
    ops::{Deref, DerefMut},
    convert::TryFrom,
    iter::FromIterator,
};
use crate::{
    T3B1,
    Trits,
};

pub const TRYTE_ALPHABET: [char; 27] = [
    '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
    'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

pub trait IsTryte {
    fn is_tryte(&self) -> bool;
}

impl IsTryte for char {
    fn is_tryte(&self) -> bool {
        *self == '9' || (*self >= 'A' && *self <= 'Z')
    }
}





#[derive(Debug)]
pub enum TryteError {
    InvalidRepr,
}

// TODO: Fill out numbers properly?
#[repr(i8)]
pub enum Tryte {
    N = -13,
    O = -12,
    P = -11,
    Q = -10,
    R = -9,
    S = -8,
    T = -7,
    U = -6,
    V = -5,
    W = -4,
    X = -3,
    Y = -2,
    Z = -1,
    Nine = 0,
    A = 1,
    B = 2,
    C = 3,
    D = 4,
    E = 5,
    F = 6,
    G = 7,
    H = 8,
    I = 9,
    J = 10,
    K = 11,
    L = 12,
    M = 13,
}

impl Tryte {
    pub fn as_trits(&self) -> &Trits<T3B1> {
        unsafe { &*(T3B1::make(self as *const _ as *const _, 0, 1) as *const _) }
    }

    pub fn as_trits_mut(&mut self) -> &mut Trits<T3B1> {
        unsafe { &mut *(T3B1::make(self as *const _ as *const _, 0, 1) as *mut _) }
    }
}

impl Into<char> for Tryte {
    fn into(self) -> char {
        TRYTE_ALPHABET[self as usize]
    }
}

impl TryFrom<char> for Tryte {
    type Error = TryteError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        if c as u8 >= b'A' && c as u8 <= b'Z' {
            // TODO: Don't do this
            Ok(unsafe { std::mem::transmute(1 + c as u8 - b'A') })
        } else if c == '9' {
            Ok(Tryte::Nine)
        } else {
            Err(TryteError::InvalidRepr)
        }
    }
}

#[derive(Default)]
pub struct TryteBuf {
    inner: Vec<Tryte>,
}

impl TryteBuf {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self { inner: Vec::with_capacity(cap) }
    }

    pub fn try_from_str(s: &str) -> Result<Self, TryteError> {
        s
            .chars()
            .map(Tryte::try_from)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn push(&mut self, tryte: Tryte) {
        self.inner.push(tryte);
    }

    pub fn pop(&mut self) -> Option<Tryte> {
        self.inner.pop()
    }

    pub fn as_trits(&self) -> &Trits<T3B1> {
        unsafe { &*(T3B1::make(self.as_ptr() as *const _, 0, self.len() * 3) as *const _) }
    }

    pub fn as_trits_mut(&mut self) -> &mut Trits<T3B1> {
        unsafe { &mut *(T3B1::make(self.as_ptr() as *const _, 0, self.len() * 3) as *mut _) }
    }
}

impl Deref for TryteBuf {
    type Target = [Tryte];
    fn deref(&self) -> &Self::Target { &self.inner }
}

impl DerefMut for TryteBuf {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}

impl FromIterator<Tryte> for TryteBuf {
    fn from_iter<I: IntoIterator<Item=Tryte>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut this = Self::with_capacity(iter.size_hint().0);
        for tryte in iter {
            this.push(tryte);
        }
        this
    }
}
