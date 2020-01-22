use std::convert::TryFrom;

/// An owned, mutable container of trits
#[derive(Clone, Debug, PartialEq)]
pub struct TritsBuf(pub(crate) Vec<i8>);

/// The possible values that a balanced trit can have.
pub enum ValidTrits {
    MinusOne,
    PlusOne,
    Zero,
}

impl From<ValidTrits> for i8 {
    fn from(v: ValidTrits) -> Self {
        use ValidTrits::*;

        match v {
            MinusOne => -1,
            PlusOne => 1,
            Zero => 0,
        }
    }
}

impl TritsBuf {
    /// Create a new `TritsBuf` with a number of `capacity` elements, all
    /// initialized to 0;
    pub fn with_capacity(capacity: usize) -> Self {
        Self(vec![0; capacity])
    }

    /// Return a read-only view of the buffer in form of a `Trits`.
    pub fn as_trits(&self) -> Trits<'_> {
        Trits(&self.0)
    }

    /// Return a read-write view of the buffer in form of a `TritsMut`.
    pub fn as_trits_mut(&mut self) -> TritsMut<'_> {
        TritsMut(&mut self.0)
    }

    /// Return a borrow of the inner vector wrapped by `TritsBuf`.
    pub fn inner_ref(&self) -> &Vec<i8> {
        &self.0
    }

    /// Return a mutable borrow of the inner vector wrapped by `TritsBuf`.
    pub fn inner_mut(&mut self) -> &mut Vec<i8> {
        &mut self.0
    }

    /// Return the inner vector wrapped by `TritsBuf`, dropping the `TritsBuf`.
    pub fn into_inner(self) -> Vec<i8> {
        self.0
    }

    /// Overwrite all elements in the `TritsBuf` with `v`.
    pub fn fill(&mut self, v: ValidTrits) {
        let v = v.into();
        self.0
            .iter_mut()
            .for_each(|x| *x = v);
    }

    /// Create a `Trits` from a `&[i8]` slice without verifying that its bytes are
    /// correctly binary-coded balanced trits (-1, 0, and +1).
    ///
    /// This function is intended to be used in hot loops and relies on the user making sure that
    /// the bytes are set correctly.
    ///
    /// **NOTE:** Use the `TryFrom` trait if you want to check that the slice encodes trits
    /// correctly before creating `Trits`.
    ///
    /// **WARNING:** If used incorrectly (that is, if the bytes are not correctly encoding trits), the
    /// usage of `Trits` might lead to unexpected behaviour.
    pub fn from_i8_unchecked<T: Into<Vec<i8>>>(v: T) -> Self {
        Self(v.into())
    }

    /// Create a `Trits` from a `&[u8]` slice without verifying that its bytes are
    /// correctly binary-coded balanced trits (-1, 0, and +1 transmuted to unsigned bytes).
    ///
    /// This function is intended to be used in hot loops and relies on the user making sure that
    /// the bytes are set correctly.
    ///
    /// **NOTE:** Use the `TryFrom` trait if you want to check that the slice encodes trits
    /// correctly before creating `Trits`.
    ///
    /// **WARNING:** If used incorrectly (that is, if the bytes are not correctly encoding trits), the
    /// usage of `Trits` might lead to unexpected behaviour.
    pub fn from_u8_unchecked<T: Into<Vec<u8>>>(v: T) -> Self {
        let inner = v.into();
        let mut inner = std::mem::ManuallyDrop::new(inner);

        let p = inner.as_mut_ptr();
        let len = inner.len();
        let cap = inner.capacity();

        let reconstructed = unsafe {
            let p_as_i8 = p as *mut i8;
            Vec::from_raw_parts(p_as_i8, len, cap)
        };
        Self::from_i8_unchecked(reconstructed)
    }
}

impl TryFrom<Vec<i8>> for TritsBuf {
    type Error = FromI8Error;

    fn try_from(vs: Vec<i8>) -> Result<Self, Self::Error> {
        for v in &vs {
            match v {
                0 | -1 | 1 => {},
                _ => Err(FromI8Error)?,
            }
        }
        Ok(TritsBuf::from_i8_unchecked(vs))
    }
}

impl TryFrom<Vec<u8>> for TritsBuf {
    type Error = FromU8Error;

    fn try_from(vs: Vec<u8>) -> Result<Self, Self::Error> {
        for v in &vs {
            match v {
                0b0000_0000 | 0b1111_1111 | 0b0000_0001 => {},
                _ => Err(FromU8Error)?,
            }
        }

        Ok(Self::from_u8_unchecked(vs))
    }
}

#[derive(Debug, PartialEq)]
pub struct Trits<'a>(pub(crate) &'a [i8]);

#[derive(Debug, PartialEq)]
pub struct TritsMut<'a>(pub(crate) &'a mut [i8]);

pub struct FromU8Error;
pub struct FromI8Error;

/// Similar impls for `TritsMut` and `TritsBuf`
impl<'a> Trits<'a> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Create a `Trits` from a `&[i8]` slice without verifying that its bytes are
    /// correctly binary-coded balanced trits (-1, 0, and +1).
    ///
    /// This function is intended to be used in hot loops and relies on the user making sure that
    /// the bytes are set correctly.
    ///
    /// **NOTE:** Use the `TryFrom` trait if you want to check that the slice encodes trits
    /// correctly before creating `Trits`.
    ///
    /// **WARNING:** If used incorrectly (that is, if the bytes are not correctly encoding trits), the
    /// usage of `Trits` might lead to unexpected behaviour.
    pub fn from_i8_unchecked(v: &'a [i8]) -> Self {
        Self(v)
    }

    /// Create a `Trits` from a `&[u8]` slice without verifying that its bytes are
    /// correctly binary-coded balanced trits (-1, 0, and +1 transmuted to unsigned bytes).
    ///
    /// This function is intended to be used in hot loops and relies on the user making sure that
    /// the bytes are set correctly.
    ///
    /// **NOTE:** Use the `TryFrom` trait if you want to check that the slice encodes trits
    /// correctly before creating `Trits`.
    ///
    /// **WARNING:** If used incorrectly (that is, if the bytes are not correctly encoding trits), the
    /// usage of `Trits` might lead to unexpected behaviour.
    pub fn from_u8_unchecked(v: &[u8]) -> Self {
        Self::from_i8_unchecked(
            unsafe {
                &*(v as *const _ as *const [i8])
        })
    }

    /// Return a borrow of the inner slice wrapped by `Trits`.
    pub fn inner_ref(&self) -> &[i8] {
        self.0
    }
}

impl<'a> TryFrom<&'a [u8]> for Trits<'a> {
    type Error = FromU8Error;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        for byte in v {
            match byte {
                0b0000_0000 | 0b1111_1111 | 0b0000_0001 => {},
                _ => Err(FromU8Error)?,
            }
        }

        Ok( Self::from_u8_unchecked(v) )
    }
}

impl<'a> TryFrom<&'a [i8]> for Trits<'a> {
    type Error = FromI8Error;

    fn try_from(v: &'a [i8]) -> Result<Self, Self::Error> {
        for byte in v {
            match byte {
                0 | -1 | 1 => {},
                _ => Err(FromI8Error)?,
            }
        }

        Ok( Self::from_i8_unchecked(v) )
    }
}

impl<'a> TritsMut<'a> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn from_i8_unchecked(v: &'a mut [i8]) -> Self {
        Self(v)
    }

    pub fn from_u8_unchecked(v: &mut [u8]) -> Self {
        Self::from_i8_unchecked(
            unsafe {
                &mut *(v as *mut _ as *mut [i8])
        })
    }

    /// Return a borrow of the inner slice wrapped by `Trits`.
    pub fn inner_ref(&self) -> &[i8] {
        self.0
    }

    pub fn inner_mut(&mut self) -> &mut [i8] {
        self.0
    }
}

impl<'a> TryFrom<&'a mut [i8]> for TritsMut<'a> {
    type Error = FromI8Error;

    fn try_from(v: &'a mut [i8]) -> Result<Self, Self::Error> {
        for byte in v.iter() {
            match byte {
                0 | -1 | 1 => {},
                _ => Err(FromI8Error)?,
            }
        }

        Ok( Self::from_i8_unchecked(v) )
    }
}


impl<'a> TryFrom<&'a mut [u8]> for TritsMut<'a> {
    type Error = FromU8Error;

    fn try_from(v: &mut [u8]) -> Result<Self, Self::Error> {
        for byte in v.iter() {
            match byte {
                0b0000_0000 | 0b1111_1111 | 0b0000_0001 => {},
                _ => Err(FromU8Error)?,
            }
        }

        Ok( Self::from_u8_unchecked(v) )
    }
}
