use byteorder::{
    self,
    ByteOrder,
};

use std::cmp::Ordering;
use std::convert::{
    TryFrom,
    TryInto,
};
use std::fmt;
use std::marker::PhantomData;

use crate::{
    bigint::{
        common::{
            BigEndian,
            BinaryRepresentation,
            Error,
            LittleEndian,
            U32Repr,
            U8Repr,
            BINARY_LEN_IN_U32 as LEN_IN_U32,
            BINARY_LEN_IN_U8 as LEN_IN_U8,
        },
        u384,
        utils::OverflowingAddExt,
        T242,
        T243,
        U384,
    },
    Btrit,
};

mod constants;
pub use constants::{
    BE_U32_0,
    BE_U32_1,
    BE_U32_2,
    BE_U32_MAX,
    BE_U32_MIN,
    BE_U32_NEG_1,
    BE_U32_NEG_2,
    BE_U8_0,
    BE_U8_1,
    BE_U8_2,
    BE_U8_MAX,
    BE_U8_MIN,
    BE_U8_NEG_1,
    BE_U8_NEG_2,
    LE_U32_0,
    LE_U32_1,
    LE_U32_2,
    LE_U32_MAX,
    LE_U32_MIN,
    LE_U32_NEG_1,
    LE_U32_NEG_2,
    LE_U8_0,
    LE_U8_1,
    LE_U8_2,
    LE_U8_MAX,
    LE_U8_MIN,
    LE_U8_NEG_1,
    LE_U8_NEG_2,
};

/// A biginteger encoding a signed integer with 384 bits.
///
/// `T` is usually taken as a `[u32; 12]` or `[u8; 48]`.
///
/// `E` refers to the endianness of the digits in `T`. This means that in the case of `[u32; 12]`,
/// if `E == BigEndian`, that the u32 at position i=0 is considered the most significant digit. The
/// endianness `E` here makes no statement about the endianness of each single digit within itself
/// (this then is dependent on the endianness of the platform this code is run on).
///
/// For `E == LittleEndian` the digit at the last position is considered to be the most
/// significant.
#[derive(Clone, Copy)]
pub struct I384<E, T> {
    pub(crate) inner: T,
    _phantom: PhantomData<E>,
}

impl<E, T> I384<E, T> {
    pub fn inner_ref(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<E: fmt::Debug, R: BinaryRepresentation, D> fmt::Debug for I384<E, R>
where
    E: fmt::Debug,
    R: BinaryRepresentation<T = D>,
    D: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("I384")
            .field("inner", &self.inner.iter())
            .field("_phantom", &self._phantom)
            .finish()
    }
}

impl_const_functions!(
    ( I384 ),
    { BigEndian, LittleEndian },
    { U8Repr, U32Repr }
);

impl_constants!(
    I384<BigEndian, U8Repr> => [
        (zero, BE_U8_0),
        (one, BE_U8_1),
        (neg_one, BE_U8_NEG_1),
        (two, BE_U8_2),
        (neg_two, BE_U8_NEG_2),
        (max, BE_U8_MAX),
        (min, BE_U8_MIN),
    ],
    I384<LittleEndian, U8Repr> => [
        (zero, LE_U8_0),
        (one, LE_U8_1),
        (neg_one, LE_U8_NEG_1),
        (two, LE_U8_2),
        (neg_two, LE_U8_NEG_2),
        (max, LE_U8_MAX),
        (min, LE_U8_MIN),
    ],
    I384<BigEndian, U32Repr> => [
        (zero, BE_U32_0),
        (one, BE_U32_1),
        (neg_one, BE_U32_NEG_1),
        (two, BE_U32_2),
        (neg_two, BE_U32_NEG_2),
        (max, BE_U32_MAX),
        (min, BE_U32_MIN),
    ],
    I384<LittleEndian, U32Repr> => [
        (zero, LE_U32_0),
        (one, LE_U32_1),
        (neg_one, LE_U32_NEG_1),
        (two, LE_U32_2),
        (neg_two, LE_U32_NEG_2),
        (max, LE_U32_MAX),
        (min, LE_U32_MIN),
    ],
);

macro_rules! impl_default {
    ( ( $($type:tt)* ), $len:expr ) => {
        impl Default for $($type)* {
            fn default() -> Self {
                Self {
                    inner: [0; $len],
                    _phantom: PhantomData,
                }
            }
        }
    };
}

impl I384<BigEndian, U8Repr> {
    pub fn not_inplace(&mut self) {
        for digit in &mut self.inner[..] {
            *digit = !*digit;
        }
    }
}

impl_default!((I384<BigEndian, U8Repr>), LEN_IN_U8);

impl Eq for I384<BigEndian, U8Repr> {}

impl From<I384<BigEndian, U32Repr>> for I384<BigEndian, U8Repr> {
    fn from(value: I384<BigEndian, U32Repr>) -> Self {
        let mut i384_u8 = Self::zero();
        byteorder::BigEndian::write_u32_into(&value.inner, &mut i384_u8.inner);
        i384_u8
    }
}

impl PartialEq for I384<BigEndian, U8Repr> {
    fn eq(&self, other: &Self) -> bool {
        let mut are_equal = true;
        for (a, b) in self.inner.iter().zip(other.inner.iter()) {
            are_equal &= a == b
        }
        are_equal
    }
}

impl PartialOrd for I384<BigEndian, U8Repr> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use Ordering::*;

        let self_iter = self.inner.iter();
        let other_iter = other.inner.iter();

        let mut zipped_iter = self_iter.zip(other_iter);

        // The most significant u8 (MSU8) has to be handled separately.
        //
        // If the most significant bit of both numbers is set, then the comparison operators
        // have to be reversed.
        //
        // Note that this is only relevant to the comparison operators between the less significant
        // u8 if the two MSU8s are equal. If they are not equal, then an early return will be
        // triggered.

        const UMAX: u8 = std::u8::MAX;
        let numbers_negative = match zipped_iter.next() {
            // Case 1: both numbers are negative, s is less
            Some((s @ 0x70..=UMAX, o @ 0x70..=UMAX)) if s > o => return Some(Less),

            // Case 2: both numbers are negative, s is greater
            Some((s @ 0x70..=UMAX, o @ 0x70..=UMAX)) if s < o => return Some(Greater),

            // Case 3: both numbers are negative, but equal
            Some((0x70..=UMAX, 0x70..=UMAX)) => true,

            // Case 4: only s is negative
            Some((0x70..=UMAX, _)) => return Some(Less),

            // Case 5: only o is negative
            Some((_, 0x70..=UMAX)) => return Some(Greater),

            // Case 6: both are positive
            Some((s, o)) if s > o => return Some(Greater),

            Some((s, o)) if s < o => return Some(Less),

            // Fallthrough case; only happens if s == o
            Some(_) => false,

            // The array inside `I384` always has a length larger zero, so the first element is
            // guaranteed to exist.
            None => unreachable!(),
        };

        // Create two separate loops as to avoid repeatedly checking `numbers_negative`.
        if numbers_negative {
            for (s, o) in zipped_iter {
                if s > o {
                    return Some(Less);
                } else if s < o {
                    return Some(Greater);
                }
            }
        } else {
            for (s, o) in zipped_iter {
                if s > o {
                    return Some(Greater);
                } else if s < o {
                    return Some(Less);
                }
            }
        }

        Some(Equal)
    }
}

impl Ord for I384<BigEndian, U8Repr> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.partial_cmp(other) {
            Some(ordering) => ordering,

            // The ordering is total, hence `partial_cmp` will never return `None`.
            None => unreachable!(),
        }
    }
}

impl From<T242<Btrit>> for I384<BigEndian, U8Repr> {
    fn from(value: T242<Btrit>) -> Self {
        let i384_le: I384<LittleEndian, U32Repr> = value.into();
        let i384_be: I384<BigEndian, U32Repr> = i384_le.into();
        i384_be.into()
    }
}

impl TryFrom<T243<Btrit>> for I384<BigEndian, U8Repr> {
    type Error = Error;

    fn try_from(value: T243<Btrit>) -> Result<Self, Error> {
        let i384_le: I384<LittleEndian, U32Repr> = value.try_into()?;
        let i384_be: I384<BigEndian, U32Repr> = i384_le.into();
        Ok(i384_be.into())
    }
}

impl I384<BigEndian, U32Repr> {
    /// Adds `other` onto `self` in place.
    pub fn add_inplace(&mut self, other: Self) {
        let mut overflown = false;
        let self_iter = self.inner.iter_mut().rev();
        let other_iter = other.inner.iter().rev();

        for (s, o) in self_iter.zip(other_iter) {
            let (sum, still_overflown) = s.overflowing_add_with_carry(*o, overflown as u32);
            *s = sum;
            overflown = still_overflown;
        }
    }

    /// Adds `other` in place, returning the number of digits required accomodate `other` (starting
    /// from the least significant one).
    pub fn add_integer_inplace<T: Into<u32>>(&mut self, other: T) -> usize {
        let other = other.into();

        let (sum, mut overflown) = self.inner[0].overflowing_add(other);
        self.inner[0] = sum;

        let mut i = self.inner.len() - 1;

        while overflown {
            let (sum, still_overflown) = self.inner[i].overflowing_add(1u32);
            self.inner[i] = sum;
            overflown = still_overflown;
            i -= 1;
        }

        i
    }

    pub fn as_u384(self) -> U384<BigEndian, U32Repr> {
        U384::<BigEndian, U32Repr>::from_array(self.inner)
    }

    pub fn from_t242(value: T242<Btrit>) -> Self {
        // First make it unbalanced.
        let t242_unbalanced = value.into_shifted();

        // Then expand the size.
        let t243_unbalanced = t242_unbalanced.into_t243();

        // Unwrapping here is ok because a ut242 always fits into a u384
        let mut u384_integer = U384::<BigEndian, U32Repr>::try_from_t243(t243_unbalanced).unwrap();
        u384_integer.sub_inplace(*u384::BE_U32_HALF_MAX_T242);
        u384_integer.as_i384()
    }

    pub fn is_positive(&self) -> bool {
        (self.inner[LEN_IN_U32 - 1] & 0x7000_0000) == 0x7000_0000
    }

    pub fn is_negative(&self) -> bool {
        !self.is_positive()
    }

    /// Applies logical not to all elements in a `&[u32]`, modfiying them in place.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let xs: I384<LittleEndian, _> = I384::from_array([0xffff_ffffu32; LEN_IN_U32]);
    /// let mut ys = I384::from_array([0x0000_0000u32; LEN_IN_U32]);
    /// ys.not_inplace();
    /// assert_eq!(xs, ys);
    /// ```
    pub fn not_inplace(&mut self) {
        for i in self.inner.iter_mut() {
            *i = !*i;
        }
    }

    pub fn shift_into_u384(self) -> U384<BigEndian, U32Repr> {
        let mut u384_value = self.as_u384();
        u384_value.sub_inplace(*u384::BE_U32_HALF_MAX);
        u384_value.sub_inplace(U384::<BigEndian, U32Repr>::one());
        u384_value
    }

    /// Subtract `other` from `self` inplace.
    ///
    /// This function is defined in terms of `overflowing_add` by making use of the following identity
    /// (in terms of Two's complement, and where `!` is logical bitwise negation):
    ///
    /// !x = -x -1 => -x = !x + 1
    ///
    /// TODO: Verifiy that the final assert is indeed not necessary. Preliminary testing shows that
    /// results are as expected.
    pub fn sub_inplace(&mut self, other: Self) {
        let self_iter = self.inner.iter_mut().rev();
        let other_iter = other.inner.iter().rev();

        // The first `borrow` is always true because the addition operation needs to account for the
        // above).
        let mut borrow = true;

        for (s, o) in self_iter.zip(other_iter) {
            let (sum, has_overflown) = s.overflowing_add_with_carry(!*o, borrow as u32);
            *s = sum;
            borrow = has_overflown;
        }
    }

    /// Subtracts `other` in place, returning the number of digits required accomodate `other`
    /// (starting from the least significant one).
    pub fn sub_integer_inplace<T: Into<u32>>(&mut self, other: T) -> usize {
        let other = other.into();

        let (sum, mut overflown) = self.inner[0].overflowing_sub(other);
        self.inner[0] = sum;

        let mut i = self.inner.len() - 1;

        while overflown {
            let (sum, still_overflown) = self.inner[i].overflowing_sub(1u32);
            self.inner[i] = sum;
            overflown = still_overflown;
            i -= 1;
        }
        i
    }

    pub fn try_from_t243(balanced_trits: T243<Btrit>) -> Result<Self, Error> {
        let unbalanced_trits = balanced_trits.into_shifted();
        let u384_integer = U384::<BigEndian, U32Repr>::try_from_t243(unbalanced_trits)?;
        Ok(u384_integer.shift_into_i384())
    }
}

impl_default!((I384<BigEndian, U32Repr>), LEN_IN_U32);

impl Eq for I384<BigEndian, U32Repr> {}

impl From<I384<BigEndian, U8Repr>> for I384<BigEndian, U32Repr> {
    fn from(value: I384<BigEndian, U8Repr>) -> Self {
        let mut i384_u32 = Self::zero();
        byteorder::BigEndian::read_u32_into(&value.inner, &mut i384_u32.inner);
        i384_u32
    }
}

impl Ord for I384<BigEndian, U32Repr> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.partial_cmp(other) {
            Some(ordering) => ordering,

            // The ordering is total, hence `partial_cmp` will never return `None`.
            None => unreachable!(),
        }
    }
}

impl PartialEq for I384<BigEndian, U32Repr> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl PartialOrd for I384<BigEndian, U32Repr> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use Ordering::*;

        let self_iter = self.inner.iter();
        let other_iter = other.inner.iter();

        let mut zipped_iter = self_iter.zip(other_iter);

        // The most significant u32 (MSU32) has to be handled separately.
        //
        // If the most significant bit of both numbers is set, then the comparison operators
        // have to be reversed.
        //
        // Note that this is only relevant to the comparison operators between the less significant
        // u32 if the two MSU32s are equal. If they are not equal, then an early return will be
        // triggered.

        const UMAX: u32 = std::u32::MAX;
        let numbers_negative = match zipped_iter.next() {
            // Case 1: both numbers are negative, s is less
            Some((s @ 0x7000_0000..=UMAX, o @ 0x7000_0000..=UMAX)) if s > o => return Some(Less),

            // Case 2: both numbers are negative, s is greater
            Some((s @ 0x7000_0000..=UMAX, o @ 0x7000_0000..=UMAX)) if s < o => return Some(Greater),

            // Case 3: both numbers are negative, but equal
            Some((0x7000_0000..=UMAX, 0x7000_0000..=UMAX)) => true,

            // Case 4: only s is negative
            Some((0x7000_0000..=UMAX, _)) => return Some(Less),

            // Case 5: only o is negative
            Some((_, 0x7000_0000..=UMAX)) => return Some(Greater),

            // Case 6: both are positive
            Some((s, o)) if s > o => return Some(Greater),

            Some((s, o)) if s < o => return Some(Less),

            // Fallthrough case; only happens if s == o
            Some(_) => false,

            // The array inside `I384` always has a length larger zero, so the first element is
            // guaranteed to exist.
            None => unreachable!(),
        };

        // Create two separate loops as to avoid repeatedly checking `numbers_negative`.
        if numbers_negative {
            for (s, o) in zipped_iter {
                if s > o {
                    return Some(Less);
                } else if s < o {
                    return Some(Greater);
                }
            }
        } else {
            for (s, o) in zipped_iter {
                if s > o {
                    return Some(Greater);
                } else if s < o {
                    return Some(Less);
                }
            }
        }

        Some(Equal)
    }
}

impl From<T242<Btrit>> for I384<BigEndian, U32Repr> {
    fn from(value: T242<Btrit>) -> Self {
        let i384_le: I384<LittleEndian, U32Repr> = value.into();
        i384_le.into()
    }
}

impl TryFrom<T243<Btrit>> for I384<BigEndian, U32Repr> {
    type Error = Error;

    fn try_from(value: T243<Btrit>) -> Result<Self, Error> {
        let i384_le: I384<LittleEndian, U32Repr> = value.try_into()?;
        Ok(i384_le.into())
    }
}

impl_default!((I384<LittleEndian, U8Repr>), LEN_IN_U8);

impl Eq for I384<LittleEndian, U8Repr> {}

impl From<I384<LittleEndian, U32Repr>> for I384<LittleEndian, U8Repr> {
    fn from(value: I384<LittleEndian, U32Repr>) -> Self {
        let mut i384_u8 = Self::zero();
        byteorder::LittleEndian::write_u32_into(&value.inner, &mut i384_u8.inner);
        i384_u8
    }
}

impl Ord for I384<LittleEndian, U8Repr> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.partial_cmp(other) {
            Some(ordering) => ordering,

            // The ordering is total, hence `partial_cmp` will never return `None`.
            None => unreachable!(),
        }
    }
}

impl PartialEq for I384<LittleEndian, U8Repr> {
    fn eq(&self, other: &Self) -> bool {
        let mut are_equal = true;
        for (a, b) in self.inner.iter().zip(other.inner.iter()) {
            are_equal &= a == b
        }
        are_equal
    }
}

impl PartialOrd for I384<LittleEndian, U8Repr> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use Ordering::*;

        let self_iter = self.inner.iter().rev();
        let other_iter = other.inner.iter().rev();

        let mut zipped_iter = self_iter.zip(other_iter);

        // The most significant u8 (MSU8) has to be handled separately.
        //
        // If the most significant bit of both numbers is set, then the comparison operators
        // have to be reversed.
        //
        // Note that this is only relevant to the comparison operators between the less significant
        // u8 if the two MSU8s are equal. If they are not equal, then an early return will be
        // triggered.

        const UMAX: u8 = std::u8::MAX;
        let numbers_negative = match zipped_iter.next() {
            // Case 1: both numbers are negative, s is less
            Some((s @ 0x70..=UMAX, o @ 0x70..=UMAX)) if s > o => return Some(Less),

            // Case 2: both numbers are negative, s is greater
            Some((s @ 0x70..=UMAX, o @ 0x70..=UMAX)) if s < o => return Some(Greater),

            // Case 3: both numbers are negative, but equal
            Some((0x70..=UMAX, 0x70..=UMAX)) => true,

            // Case 4: only s is negative
            Some((0x70..=UMAX, _)) => return Some(Less),

            // Case 5: only o is negative
            Some((_, 0x70..=UMAX)) => return Some(Greater),

            // Case 6: both are positive
            Some((s, o)) if s > o => return Some(Greater),

            Some((s, o)) if s < o => return Some(Less),

            // Fallthrough case; only happens if s == o
            Some(_) => false,

            // The array inside `I384` always has a length larger zero, so the first element is
            // guaranteed to exist.
            None => unreachable!(),
        };

        // Create two separate loops as to avoid repeatedly checking `numbers_negative`.
        if numbers_negative {
            for (s, o) in zipped_iter {
                if s > o {
                    return Some(Less);
                } else if s < o {
                    return Some(Greater);
                }
            }
        } else {
            for (s, o) in zipped_iter {
                if s > o {
                    return Some(Greater);
                } else if s < o {
                    return Some(Less);
                }
            }
        }

        Some(Equal)
    }
}

impl I384<LittleEndian, U32Repr> {
    /// Adds `other` onto `self` in place.
    pub fn add_inplace(&mut self, other: Self) {
        let mut overflown = false;
        let self_iter = self.inner.iter_mut();
        let other_iter = other.inner.iter();

        for (s, o) in self_iter.zip(other_iter) {
            let (sum, still_overflown) = s.overflowing_add_with_carry(*o, overflown as u32);
            *s = sum;
            overflown = still_overflown;
        }
    }

    /// Adds `other` in place, returning the number of digits required accomodate `other` (starting
    /// from the least significant one).
    pub fn add_integer_inplace<T: Into<u32>>(&mut self, other: T) -> usize {
        let other = other.into();

        let (sum, mut overflown) = self.inner[0].overflowing_add(other);
        self.inner[0] = sum;

        let mut i = 1;

        while overflown {
            let (sum, still_overflown) = self.inner[i].overflowing_add(1u32);
            self.inner[i] = sum;
            overflown = still_overflown;
            i += 1;
        }

        i
    }

    pub fn as_u384(self) -> U384<LittleEndian, U32Repr> {
        U384::<LittleEndian, U32Repr>::from_array(self.inner)
    }

    pub fn from_t242(value: T242<Btrit>) -> Self {
        // First make it unbalanced.
        let t242_unbalanced = value.into_shifted();

        // Then expand the size.
        let t243_unbalanced = t242_unbalanced.into_t243();

        // Unwrapping here is okay, because a ut242 always fits into a u384
        let mut u384_integer = U384::<LittleEndian, U32Repr>::try_from_t243(t243_unbalanced).unwrap();
        u384_integer.sub_inplace(*u384::LE_U32_HALF_MAX_T242);
        u384_integer.as_i384()
    }

    pub fn is_positive(&self) -> bool {
        (self.inner[LEN_IN_U32 - 1] & 0x7000_0000) == 0x7000_0000
    }

    pub fn is_negative(&self) -> bool {
        !self.is_positive()
    }

    /// Applies logical not to all elements in a `&[u32]`, modfiying them in place.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let xs: I384<LittleEndian, _> = I384::from_array([0xffff_ffffu32; LEN_IN_U32]);
    /// let mut ys = I384::from_array([0x0000_0000u32; LEN_IN_U32]);
    /// ys.not_inplace();
    /// assert_eq!(xs, ys);
    /// ```
    pub fn not_inplace(&mut self) {
        for i in self.inner.iter_mut() {
            *i = !*i;
        }
    }

    pub fn shift_into_u384(self) -> U384<LittleEndian, U32Repr> {
        let mut u384_value = self.as_u384();
        u384_value.sub_inplace(*u384::LE_U32_HALF_MAX);
        u384_value.sub_inplace(U384::<LittleEndian, U32Repr>::one());
        u384_value
    }

    /// Subtract `other` from `self` inplace.
    ///
    /// This function is defined in terms of `overflowing_add` by making use of the following identity
    /// (in terms of Two's complement, and where `!` is logical bitwise negation):
    ///
    /// !x = -x -1 => -x = !x + 1
    ///
    /// TODO: Verifiy that the final assert is indeed not necessary. Preliminary testing shows that
    /// results are as expected.
    pub fn sub_inplace(&mut self, other: Self) {
        let self_iter = self.inner.iter_mut();
        let other_iter = other.inner.iter();

        // The first `borrow` is always true because the addition operation needs to account for the
        // above).
        let mut borrow = true;

        for (s, o) in self_iter.zip(other_iter) {
            let (sum, has_overflown) = s.overflowing_add_with_carry(!*o, borrow as u32);
            *s = sum;
            borrow = has_overflown;
        }
    }

    /// Subtracts `other` in place, returning the number of digits required accomodate `other`
    /// (starting from the least significant one).
    pub fn sub_integer_inplace<T: Into<u32>>(&mut self, other: T) -> usize {
        let other = other.into();

        let (sum, mut overflown) = self.inner[0].overflowing_sub(other);
        self.inner[0] = sum;

        let mut i = 1;

        while overflown {
            let (sum, still_overflown) = self.inner[i].overflowing_sub(1u32);
            self.inner[i] = sum;
            overflown = still_overflown;
            i += 1;
        }
        i
    }

    pub fn try_from_t243(balanced_trits: T243<Btrit>) -> Result<Self, Error> {
        let unbalanced_trits = balanced_trits.into_shifted();
        let u384_integer = U384::<LittleEndian, U32Repr>::try_from_t243(unbalanced_trits)?;
        Ok(u384_integer.shift_into_i384())
    }
}

impl_default!((I384<LittleEndian, U32Repr>), LEN_IN_U32);

impl Eq for I384<LittleEndian, U32Repr> {}

impl From<I384<LittleEndian, U8Repr>> for I384<LittleEndian, U32Repr> {
    fn from(value: I384<LittleEndian, U8Repr>) -> Self {
        let mut i384_u32 = I384::<LittleEndian, U32Repr>::zero();
        byteorder::LittleEndian::read_u32_into(&value.inner, &mut i384_u32.inner);
        i384_u32
    }
}

impl Ord for I384<LittleEndian, U32Repr> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.partial_cmp(other) {
            Some(ordering) => ordering,

            // The ordering is total, hence `partial_cmp` will never return `None`.
            None => unreachable!(),
        }
    }
}

impl PartialEq for I384<LittleEndian, U32Repr> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl PartialOrd for I384<LittleEndian, U32Repr> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use Ordering::*;

        let self_iter = self.inner.iter().rev();
        let other_iter = other.inner.iter().rev();

        let mut zipped_iter = self_iter.zip(other_iter);

        // The most significant u32 (MSU32) has to be handled separately.
        //
        // If the most significant bit of both numbers is set, then the comparison operators
        // have to be reversed.
        //
        // Note that this is only relevant to the comparison operators between the less significant
        // u32 if the two MSU32s are equal. If they are not equal, then an early return will be
        // triggered.

        const UMAX: u32 = std::u32::MAX;
        let numbers_negative = match zipped_iter.next() {
            // Case 1: both numbers are negative, s is less
            Some((s @ 0x7000_0000..=UMAX, o @ 0x7000_0000..=UMAX)) if s > o => return Some(Less),

            // Case 2: both numbers are negative, s is greater
            Some((s @ 0x7000_0000..=UMAX, o @ 0x7000_0000..=UMAX)) if s < o => return Some(Greater),

            // Case 3: both numbers are negative, but equal
            Some((0x7000_0000..=UMAX, 0x7000_0000..=UMAX)) => true,

            // Case 4: only s is negative
            Some((0x7000_0000..=UMAX, _)) => return Some(Less),

            // Case 5: only o is negative
            Some((_, 0x7000_0000..=UMAX)) => return Some(Greater),

            // Case 6: both are positive
            Some((s, o)) if s > o => return Some(Greater),

            Some((s, o)) if s < o => return Some(Less),

            // Fallthrough case; only happens if s == o
            Some(_) => false,

            // The array inside `I384` always has a length larger zero, so the first element is
            // guaranteed to exist.
            None => unreachable!(),
        };

        // Create two separate loops as to avoid repeatedly checking `numbers_negative`.
        if numbers_negative {
            for (s, o) in zipped_iter {
                if s > o {
                    return Some(Less);
                } else if s < o {
                    return Some(Greater);
                }
            }
        } else {
            for (s, o) in zipped_iter {
                if s > o {
                    return Some(Greater);
                } else if s < o {
                    return Some(Less);
                }
            }
        }

        Some(Equal)
    }
}

impl From<T242<Btrit>> for I384<LittleEndian, U32Repr> {
    fn from(value: T242<Btrit>) -> Self {
        Self::from_t242(value)
    }
}

impl TryFrom<T243<Btrit>> for I384<LittleEndian, U32Repr> {
    type Error = Error;
    fn try_from(value: T243<Btrit>) -> Result<Self, Error> {
        Self::try_from_t243(value)
    }
}

impl_toggle_endianness!((I384), U8Repr, U32Repr);

#[cfg(test)]
mod tests {
    use super::*;

    test_binary_op!(
        [min_minus_one_is_max, sub_inplace, LE_U32_MIN, LE_U32_1, LE_U32_MAX],
        [
            min_plus_neg_one_is_max,
            add_inplace,
            LE_U32_MIN,
            LE_U32_NEG_1,
            LE_U32_MAX
        ],
        [min_minus_zero_is_min, sub_inplace, LE_U32_MIN, LE_U32_0, LE_U32_MIN],
        [min_plus_zero_is_min, add_inplace, LE_U32_MIN, LE_U32_0, LE_U32_MIN],
        [
            neg_one_minus_one_is_neg_two,
            sub_inplace,
            LE_U32_NEG_1,
            LE_U32_1,
            LE_U32_NEG_2
        ],
        [
            neg_one_minus_neg_one_is_zero,
            sub_inplace,
            LE_U32_NEG_1,
            LE_U32_NEG_1,
            LE_U32_0
        ],
        [neg_one_plus_one_is_zero, add_inplace, LE_U32_NEG_1, LE_U32_1, LE_U32_0],
        [
            neg_one_plus_neg_one_is_neg_two,
            add_inplace,
            LE_U32_NEG_1,
            LE_U32_NEG_1,
            LE_U32_NEG_2
        ],
        [zero_minus_one_is_neg_one, sub_inplace, LE_U32_0, LE_U32_1, LE_U32_NEG_1],
        [zero_minus_neg_one_is_one, sub_inplace, LE_U32_0, LE_U32_NEG_1, LE_U32_1],
        [zero_plus_one_is_one, add_inplace, LE_U32_0, LE_U32_1, LE_U32_1],
        [
            zero_plus_neg_one_is_neg_one,
            add_inplace,
            LE_U32_0,
            LE_U32_NEG_1,
            LE_U32_NEG_1
        ],
        [one_minus_neg_one_is_two, sub_inplace, LE_U32_1, LE_U32_NEG_1, LE_U32_2],
        [one_minus_one_is_zero, sub_inplace, LE_U32_1, LE_U32_1, LE_U32_0],
        [one_plus_one_is_two, add_inplace, LE_U32_1, LE_U32_1, LE_U32_2],
        [one_plus_neg_one_is_zero, add_inplace, LE_U32_1, LE_U32_NEG_1, LE_U32_0],
        [max_plus_one_is_min, add_inplace, LE_U32_MAX, LE_U32_1, LE_U32_MIN],
        [
            max_minus_neg_one_is_min,
            sub_inplace,
            LE_U32_MAX,
            LE_U32_NEG_1,
            LE_U32_MIN
        ],
    );

    test_binary_op_calc_result!(
        [
            min_minus_two_is_max_minus_one,
            sub_inplace,
            LE_U32_MIN,
            LE_U32_2,
            sub_inplace,
            LE_U32_MAX,
            LE_U32_1
        ],
        [
            min_plus_one_is_max_plus_two,
            add_inplace,
            LE_U32_MIN,
            LE_U32_1,
            add_inplace,
            LE_U32_MAX,
            LE_U32_2
        ],
    );

    test_endianness_toggle!((I384), [u8_repr, U8Repr], [u32_repr, U32Repr],);

    test_endianness_roundtrip!((I384), [u8_repr, U8Repr], [u32_repr, U32Repr],);

    test_repr_roundtrip!((I384), [big_endian, BigEndian], [little_endian, LittleEndian],);
}
