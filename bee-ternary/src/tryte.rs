use crate::{
    Error,
    Trits,
    T3B1,
};
use std::{
    convert::TryFrom,
    fmt,
    iter::FromIterator,
    ops::{
        Deref,
        DerefMut,
    },
};

pub const MIN_TRYTE_VALUE: i8 = -13;
pub const MAX_TRYTE_VALUE: i8 = 13;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
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
        unsafe { &*(T3B1::make(self as *const _ as *const _, 0, 3) as *const _) }
    }

    pub fn as_trits_mut(&mut self) -> &mut Trits<T3B1> {
        unsafe { &mut *(T3B1::make(self as *const _ as *const _, 0, 3) as *mut _) }
    }
}

impl From<Tryte> for char {
    fn from(tryte: Tryte) -> char {
        match tryte as i8 {
            0 => '9',
            -13..=-1 => (((tryte as i8 + 13) as u8) + b'N') as char,
            1..=13 => (((tryte as i8 - 1) as u8) + b'A') as char,
            _ => unreachable!(),
        }
    }
}

impl fmt::Debug for Tryte {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", char::from(*self))
    }
}

impl fmt::Display for Tryte {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", char::from(*self))
    }
}

impl TryFrom<char> for Tryte {
    type Error = Error;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '9' => Ok(Tryte::Nine),
            'N'..='Z' => Ok(unsafe { std::mem::transmute((c as u8 - b'N') as i8 - 13) }),
            'A'..='M' => Ok(unsafe { std::mem::transmute((c as u8 - b'A') as i8 + 1) }),
            _ => Err(Error::InvalidRepr),
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
        Self {
            inner: Vec::with_capacity(cap),
        }
    }

    pub fn try_from_str(s: &str) -> Result<Self, Error> {
        s.chars().map(Tryte::try_from).collect()
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
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for TryteBuf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl FromIterator<Tryte> for TryteBuf {
    fn from_iter<I: IntoIterator<Item = Tryte>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut this = Self::with_capacity(iter.size_hint().0);
        for tryte in iter {
            this.push(tryte);
        }
        this
    }
}

impl fmt::Debug for TryteBuf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl fmt::Display for TryteBuf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for tryte in self.iter() {
            write!(f, "{}", tryte)?;
        }
        Ok(())
    }
}
