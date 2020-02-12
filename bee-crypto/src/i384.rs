use crate::{
    t243::T243,
    utils::OverflowingAddExt,
    private,
    utils::SplitInteger,
};

use std::cmp::Ordering;
use std::marker::PhantomData;

/// The number of bits in an I384
const I384_LEN_BITS: usize = 384;

/// The number of bytes in an I384
const I384_LEN_U8: usize = I384_LEN_BITS / 8;
const I384_LEN_U32: usize = I384_LEN_BITS / 32;

type I384U32 = [u32; I384_LEN_U32];
type I384U8 = [u8; I384_LEN_U8];

/// A bigint containing the result when transforming a `t243` to binary, where the `t243` contains
/// only 0s in terms of balanaced trits (or only 1s, in terms of unbalanced, unsigned trits that
/// encode {0, 1, 2}).
///
/// So `HALF_3` is the result of the sum 1 * 3^0 + 1 * 3^1 + ... + 1 * 3^241 + 1*3^242.
const HALF_3: I384<LittleEndian, I384U32> = I384::<LittleEndian, _>::from_array([
    0xa5ce8964,
    0x9f007669,
    0x1484504f,
    0x3ade00d9,
    0x0c24486e,
    0x50979d57,
    0x79a4c702,
    0x48bbae36,
    0xa9f6808b,
    0xaa06a805,
    0xa87fabdf,
    0x5e69ebef,
]);


const BE_U32_ZERO: I384<BigEndian, I384U32> = I384::<BigEndian, _>::from_array([
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
]);

const LE_U32_ZERO: I384<LittleEndian, I384U32> = I384::<LittleEndian, _>::from_array(BE_U32_ZERO.inner);

#[derive(Clone, Copy, Debug)]
struct BigEndian {}

#[derive(Clone, Copy, Debug)]
struct LittleEndian {}

impl private::Sealed for BigEndian {}
impl private::Sealed for LittleEndian {}

trait EndianType: private::Sealed {}

impl EndianType for BigEndian {}
impl EndianType for LittleEndian {}

trait I384Representation: private::Sealed + Clone {
    fn zero() -> Self;
}

impl private::Sealed for I384U8 {}
impl private::Sealed for I384U32 {}

impl I384Representation for I384U8 {
    fn zero() -> Self {
        [0; I384_LEN_U8]
    }
}

impl I384Representation for I384U32 {
    fn zero() -> Self {
        [0; I384_LEN_U32]
    }
}

/// A biginteger encoding a signed integer with 384 bits.
///
/// `T` is usually taken as a `[u32; 12]` or `[u8; 48]`.
///
/// `E` refers to the endianness of the digits in `T`. This means that in the case of `[u32; 12]`,
/// if `E == BigEndian`, that the u32 at position i=0 is considered the most significant digit. The
/// endianness `E` here makes no statement about the endianness of each single digit`.
///
/// For `E == LittleEndian` the digit at the last position is considered to be the most
/// significant.
#[derive(Clone, Copy, Debug)]
struct I384<E, T> {
    inner: T,
    _phantom: PhantomData<E>,
}

impl Eq for I384<BigEndian, I384U32> {}

impl PartialEq for I384<BigEndian, I384U32> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl PartialOrd for I384<BigEndian, I384U32> {
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
            Some(( s @ 0x7000_0000..=UMAX, o @ 0x7000_0000..=UMAX )) if s > o => return Some(Less),

            // Case 2: both numbers are negative, s is greater
            Some(( s @ 0x7000_0000..=UMAX, o @ 0x7000_0000..=UMAX )) if s < o => return Some(Greater),

            // Case 3: both numbers are negative, but equal
            Some(( 0x7000_0000..=UMAX, 0x7000_0000..=UMAX )) => true,

            // Case 4: only s is negative
            Some(( 0x7000_0000..=UMAX, _ )) => return Some(Less),

            // Case 5: only o is negative
            Some(( _, 0x7000_0000..=UMAX )) => return Some(Greater),

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
                    return Some(Less)
                } else if s < o {
                    return Some(Greater)
                }
            }
        } else {
            for (s, o) in zipped_iter {
                if s > o {
                    return Some(Greater)
                } else if s < o {
                    return Some(Less)
                }
            }
        }

        Some(Equal)
    }
}

impl Ord for I384<BigEndian, I384U32> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.partial_cmp(other) {
            Some(ordering) => ordering,

            // The ordering is total, hence `partial_cmp` will never return `None`.
            None => unreachable!(),
        }
    }
}

impl Eq for I384<LittleEndian, I384U32> {}

impl PartialEq for I384<LittleEndian, I384U32> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl PartialOrd for I384<LittleEndian, I384U32> {
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
            Some(( s @ 0x7000_0000..=UMAX, o @ 0x7000_0000..=UMAX )) if s > o => return Some(Less),

            // Case 2: both numbers are negative, s is greater
            Some(( s @ 0x7000_0000..=UMAX, o @ 0x7000_0000..=UMAX )) if s < o => return Some(Greater),

            // Case 3: both numbers are negative, but equal
            Some(( 0x7000_0000..=UMAX, 0x7000_0000..=UMAX )) => true,

            // Case 4: only s is negative
            Some(( 0x7000_0000..=UMAX, _ )) => return Some(Less),

            // Case 5: only o is negative
            Some(( _, 0x7000_0000..=UMAX )) => return Some(Greater),

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
                    return Some(Less)
                } else if s < o {
                    return Some(Greater)
                }
            }
        } else {
            for (s, o) in zipped_iter {
                if s > o {
                    return Some(Greater)
                } else if s < o {
                    return Some(Less)
                }
            }
        }

        Some(Equal)
    }
}

impl Ord for I384<LittleEndian, I384U32> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.partial_cmp(other) {
            Some(ordering) => ordering,

            // The ordering is total, hence `partial_cmp` will never return `None`.
            None => unreachable!(),
        }
    }
}


impl<E: EndianType> I384<E, I384U32> {
    /// Returns true if zero, false if not.
    ///
    /// NOTE: This test could be performed in terms of `PartialEq`, but this only has to iterate
    /// through one array.
    fn is_zero(&self) -> bool {
        for digit in self.inner.iter() {
            if *digit != 0 {
                return false;
            }
        }
        true
    }
}

impl I384<BigEndian, I384U32> {
    const fn from_array(inner: I384U32) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl I384<LittleEndian, I384U32> {
    const fn from_array(inner: I384U32) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }

    const fn zero() -> Self {
        LE_U32_ZERO
    }

    /// Adds `other` onto `self` in place.
    fn add_inplace(&mut self, other: Self) {
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
    fn add_integer_inplace<T: Into<u32>>(&mut self, other: T) -> usize {
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

    /// Subtract `other` from `self` inplace.
    ///
    /// This function is defined in terms of `overflowing_add` by making use of the following identity
    /// (in terms of Two's complement, and where `!` is logical bitwise negation):
    ///
    /// !x = -x -1 => -x = !x + 1
    ///
    /// TODO: Verifiy that the final assert is indeed not necessary. Preliminary testing shows that
    /// results are as expected.
    fn sub_inplace(&mut self, other: Self) {
        let self_iter = self.inner.iter_mut();
        let other_iter = other.inner.iter();

        // The first `borrow` is always true because the addition operation needs to account for the
        // extra `+1` when expressing subtraction via bitwise not (see the identity in the doccomment
        // above).
        let mut borrow = true;

        for (s, o) in self_iter.zip(other_iter) {
            let (sum, has_overflown) = s.overflowing_add_with_carry(!*o, borrow as u32);
            *s = sum;
            borrow = has_overflown;
        }

        // TODO: Is this truly necessary?
        // assert!(borrow);
    }

    /// Applies logical not to all elements in a `&[u32]`, modfiying them in place.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let xs: I384<LittleEndian, _> = I384::from_array([0xffff_ffffu32; I384_LEN_U32]);
    /// let mut ys = I384::from_array([0x0000_0000u32; I384_LEN_U32]);
    /// ys.not_inplace();
    /// assert_eq!(xs, ys);
    /// ```
    fn not_inplace(&mut self) {
        for i in self.inner.iter_mut() {
            *i = !*i;
        }
    }
}

impl From<T243> for I384<LittleEndian, I384U32> {
    /// Converts a signed integer represented by the balanced trits in `t243` to the signed binary
    /// integer `i384`. `t243` is assumed to be in little endian representation, with the most
    /// significant trit being at the largest index in the array.
    ///
    /// This is done in the following steps:
    ///
    /// 1. `1` is added to all balanced trits, making them *unsigned*: `{-1, 0, 1} -> {0, 1, 2}`.
    /// 2. The `t243` are converted to base 10 and through this immediately to `i384` by calculating the sum `s`:
    ///
    /// ```ignore
    /// s = t_242 * 3^241 + t_241 * 3^240 + ...
    ///   + t_{i+1} * 3^{i} + t_i * 3^{i-1} + t_{i-1} * 3^{i-2} + ...
    ///   + t_1 * 3 + t_0
    /// ```
    ///
    /// To perform this sum efficiently, its accumulation is staggered, so that each multiplication
    /// by 3 is done in each iteration of accumulating loop. This can be understood by factoring
    /// the powers of 3 from the previous sum:
    ///
    /// ```ignore
    /// s = (...((t_242 * 3 + t_241) * 3 + t_240) * 3 + ...
    ///   +  ...((t_{i+1} * 3 + t_i) * 3 + t_{i-1}) * 3 + ...
    ///   +  ...t_1) * 3 + t_0
    /// ```
    ///
    /// Or in procedural form, with the sum being accumulated in `acc` and with the index `i`
    /// running from `[242..0`]:
    ///
    /// ```ignore
    /// acc = 0
    /// for i, trit in trits.rev():
    ///     acc := acc + trit * 3^i
    /// ```
    fn from(value: T243) -> Self {
        // This is the `Vec<i8>` inside `TritsBuf`
        let mut raw_trits_buf = value.into_inner().as_i8_slice().to_vec();

        // Shift the balanced trits from {-1, 0, 1} to {0, 1, 2}
        for element in raw_trits_buf.iter_mut() {
            *element += 1;
        }

        // The accumulator is a little endian bigint using `u32` as an internal representation
        let mut accumulator: I384<LittleEndian, I384U32> = I384::zero();
        let mut accumulator_extent = 1;

        // Iterate over all trits starting from the most significant one.
        for raw_trit in raw_trits_buf.iter().rev() {

            // Iterate over all digits in the bigint accumulator, multiplying by 3 into a `u64`.
            // Overflow is handled by taking the lower `u32` as the new digit, and the higher `u32`
            // as the carry.
            let mut carry: u32 = 0;
            for digit in accumulator.inner[0..accumulator_extent].iter_mut() {
                let new_digit = *digit as u64 * 3u64 + carry as u64;

                *digit = new_digit.lo();
                carry = new_digit.hi();
            }

            if carry != 0 {
                unsafe {
                    *accumulator.inner.get_unchecked_mut(accumulator_extent) = carry;
                }
                accumulator_extent += 1;
            }

            let new_extent = accumulator.add_integer_inplace(*raw_trit as u32);
            if new_extent > accumulator_extent {
                accumulator_extent = new_extent;
            }
        }

        // Here finally, the bigint has to be made `signed`, reverting the mapping of the balanced
        // trits, `{0, 1, 2} -> {-1, 0, 1}`.
        use Ordering::*;
        match accumulator.cmp(&HALF_3) {

            // Case 1: performing the subtraction would not cause an underflow
            Greater => accumulator.sub_inplace(HALF_3),

            // Case 2: perfoming the subtraction would cause an underflow
            Less => {
                // Simulate a wrapping sub.
                let mut tmp = HALF_3.clone();
                tmp.sub_inplace(accumulator);
                tmp.not_inplace();
                tmp.add_integer_inplace(1u32);
                accumulator.clone_from(&tmp);
            }

            // Case 3:
            Equal => {
                accumulator.clone_from(&HALF_3);
                accumulator.not_inplace();
                accumulator.add_integer_inplace(1u32);
            }
        }

        accumulator
    }
}

impl From<I384<BigEndian, I384U32>> for I384<BigEndian, I384U8> {
    fn from(value: I384<BigEndian, I384U32>) -> Self {
        let mut inner = value.inner;

        // TODO: This code might be affected by the endianness of the architecture it runs on.
        for elem in inner.iter_mut() {
            *elem = elem.to_be();
        }

        // TODO: Investigate if this can be done better (and safer) using e.g. the `bytemuck`
        // library.
        let inner = unsafe {
            std::mem::transmute::<_, I384U8>(inner)
        };


        Self {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl From<I384<LittleEndian, I384U32>> for I384<BigEndian, I384U8> {
    fn from(value: I384<LittleEndian, I384U32>) -> Self {
        let value: I384<BigEndian, I384U32> = value.into();
        value.into()
    }
}

macro_rules! impl_toggle_endianness {
    ($t:ty, $src_endian:ty, $dst_endian:ty) => {
        impl From<I384<$src_endian, $t>> for I384<$dst_endian, $t> {
            fn from(value: I384<$src_endian, $t>) -> Self {
                let mut inner = value.inner;
                inner.reverse();
                Self {
                    inner,
                    _phantom: PhantomData,
                }
            }
        }
    }
}

macro_rules! impl_all_toggle_endianness {
    ( $( $t:ty ),* ) => {
        $(
            impl_toggle_endianness!($t, LittleEndian, BigEndian);
            impl_toggle_endianness!($t, BigEndian, LittleEndian);
        )*
    }
}

impl_all_toggle_endianness!(I384U8, I384U32);

#[cfg(test)]
mod tests {
    use super::*;

    const BE_U32_1: I384<BigEndian, I384U32> = I384::<BigEndian, _>::from_array([
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0001,
    ]);

    const BE_U32_NEG_1: I384<BigEndian, I384U32> = I384::<BigEndian, _>::from_array([
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
    ]);

    const BE_U32_MAX: I384<BigEndian, I384U32> = I384::<BigEndian, _>::from_array([
        0x7fff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
    ]);

    const BE_U32_MIN: I384<BigEndian, I384U32> = I384::<BigEndian, _>::from_array([
        0x8000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
    ]);

    const BE_U32_2: I384<BigEndian, I384U32> = I384::<BigEndian, _>::from_array([
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0002,
    ]);

    const BE_U32_NEG_2: I384<BigEndian, I384U32> = I384::<BigEndian, _>::from_array([
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_fffe,
    ]);

    const LE_U32_1: I384<LittleEndian, I384U32> = I384::<LittleEndian, _>::from_array([
        0x0000_0001,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
    ]);

    const LE_U32_NEG_1: I384<LittleEndian, I384U32> = I384::<LittleEndian, _>::from_array([
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
    ]);

    const LE_U32_MAX: I384<LittleEndian, I384U32> = I384::<LittleEndian, _>::from_array([
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0x7fff_ffff,
    ]);

    const LE_U32_MIN: I384<LittleEndian, I384U32> = I384::<LittleEndian, _>::from_array([
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x8000_0000,
    ]);

    const LE_U32_2: I384<LittleEndian, I384U32> = I384::<LittleEndian, _>::from_array([
        0x0000_0002,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
    ]);

    const LE_U32_NEG_2: I384<LittleEndian, I384U32> = I384::<LittleEndian, _>::from_array([
        0xffff_fffe,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
    ]);

    macro_rules! test_binary_op{
        ( $( [$testname:ident, $binop:ident, $fst:ident, $snd:ident, $res:ident] ),+ $(,)? ) => {
            mod litteendian_binary_op {
                use super::*;

                $(
                    #[test]
                    fn $testname() {
                        let mut fst = $fst;
                        fst.$binop($snd);
                        assert_eq!(fst, $res);
                    }
                )+
            }

        }
    }

    test_binary_op!(
        [one_minus_neg_one_is_two, sub_inplace, LE_U32_1, LE_U32_NEG_1, LE_U32_2],
        [one_minus_one_is_zero, sub_inplace, LE_U32_1, LE_U32_1, LE_U32_ZERO],
        [one_plus_one_is_two, add_inplace, LE_U32_1, LE_U32_1, LE_U32_2],
        [one_plus_neg_one_is_zero, add_inplace, LE_U32_1, LE_U32_NEG_1, LE_U32_ZERO],
        [neg_one_minus_one_is_neg_two, sub_inplace, LE_U32_NEG_1, LE_U32_1, LE_U32_NEG_2],
        [neg_one_minus_neg_one_is_zero, sub_inplace, LE_U32_NEG_1, LE_U32_NEG_1, LE_U32_ZERO],
        [neg_one_plus_one_is_zero, add_inplace, LE_U32_NEG_1, LE_U32_1, LE_U32_ZERO],
        [neg_one_plus_neg_one_is_neg_two, add_inplace, LE_U32_NEG_1, LE_U32_NEG_1, LE_U32_NEG_2],
        [min_minus_one_is_max, sub_inplace, LE_U32_MIN, LE_U32_1, LE_U32_MAX],
        [min_plus_neg_one_is_max, add_inplace, LE_U32_MIN, LE_U32_NEG_1, LE_U32_MAX],
        [max_plus_one_is_min, add_inplace, LE_U32_MAX, LE_U32_1, LE_U32_MIN],
        [max_minus_neg_one_is_min, sub_inplace, LE_U32_MAX, LE_U32_NEG_1, LE_U32_MIN],
        [zero_minus_one_is_neg_one, sub_inplace, LE_U32_ZERO, LE_U32_1, LE_U32_NEG_1],
        [zero_minus_neg_one_is_one, sub_inplace, LE_U32_ZERO, LE_U32_NEG_1, LE_U32_1],
        [zero_plus_one_is_one, add_inplace, LE_U32_ZERO, LE_U32_1, LE_U32_1],
        [zero_plus_neg_one_is_neg_one, add_inplace, LE_U32_ZERO, LE_U32_NEG_1, LE_U32_NEG_1],
    );

    macro_rules! test_endian_conversion {
        ( $( [$fname:ident, $be_val:ident, $le_val:ident] ),+ $(,)? ) => {
            mod bigendian_to_littleendian {
                use super::*;

                $(
                    #[test]
                    fn $fname() {
                       let converted: I384<LittleEndian, I384U32> = $be_val.into();
                       assert_eq!(converted, $le_val);
                    }
                )+
            }

            mod littleendian_to_bigendian {
                use super::*;

                $(
                    #[test]
                    fn $fname() {
                       let converted: I384<BigEndian, I384U32> = $le_val.into();
                       assert_eq!(converted, $be_val);
                    }
                )+
            }
        }
    }

    test_endian_conversion!(
        [one_is_one, BE_U32_1, LE_U32_1],
        [zero_is_zero, BE_U32_ZERO, LE_U32_ZERO],
        [neg_one_is_neg_one, BE_U32_NEG_1, LE_U32_NEG_1],
        [max_is_max, BE_U32_MAX, LE_U32_MAX],
        [min_is_min, BE_U32_MIN, LE_U32_MIN],
        [two_is_two, BE_U32_2, LE_U32_2],
        [neg_two_is_neg_two, BE_U32_NEG_2, LE_U32_NEG_2],
    );
}
