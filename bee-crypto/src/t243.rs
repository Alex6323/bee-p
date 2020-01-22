//! This module contains logic to convert an integer encoded by 243 trits to the same
//! integer encoded by 384 bits (or 48 signed bytes, `i8`).
//!
//! At the core of this a slice of binary-coded, balanced trits is interpreted
//! fanned-out `t243`, where `t243` is used analogous to `i64` or `u64`. If the latter
//! are 64-bit signed/unsigned integer types, then `t243` is a 243-trit integer type.
//! Analogous to fanning out a `u64` into 64 individual bits, `t243` is fanned out into
//! 243 trits, each (rather inefficiently) represented by one `u8`.

use crate::{
    TritsBuf,
    ValidTrits,
};
// use std::cmp::Ordering;

/// The number of trits in a T243
const T243_LEN: usize = 243;

pub(crate) struct T243(TritsBuf);

impl T243 {
    fn zero() -> Self {
        let mut inner = TritsBuf::with_capacity(T243_LEN);
        inner.fill(ValidTrits::Zero);
        Self(inner)
    }

    pub fn into_inner(self) -> TritsBuf {
        self.0
    }
}

impl Default for T243 {
    fn default() -> Self {
        Self::zero()
    }
}

// /// This will consume the input bytes slice and write to trits.
// fn bytes_to_trits(bytes: &mut [u8], trits: &mut [i8]) {
//     assert_eq!(bytes.len(), I384_LEN_BYTES);
//     assert_eq!(trits.len(), T243_LEN);

//     trits[T243_LEN - 1] = 0;

//     bytes.reverse();
//     // We _know_ that the sizes match.
//     // So this is safe enough to do and saves us a few allocations.
//     let base: &mut [u32] =
//         unsafe { core::slice::from_raw_parts_mut(bytes.as_mut_ptr() as *mut u32, 12) };

//     if base.is_zero() {
//         trits.clone_from_slice(&[0; T243_LEN]);
//         return;
//     }

//     let mut flip_trits = false;

//     if base[I384_LEN_U32 - 1] >> 31 == 0 {
//         // positive number
//         // we need to add HALF_3 to move it into positvie unsigned space
//         base.add_inplace(HALF_3);
//     } else {
//         // negative number
//         base.not_inplace();
//         if base.cmp(&HALF_3) > 0 {
//             base.sub_inplace(&HALF_3);
//             flip_trits = true;
//         } else {
//             base.add_integer_inplace(1u32);
//             let mut tmp = HALF_3.clone();
//             tmp.sub_inplace(base);
//             base.clone_from_slice(&tmp);
//         }
//     }

//     let mut rem;
//     for i in 0..T243_LEN - 1 {
//         rem = 0;
//         for j in (0..U32_IN_I384).rev() {
//             let lhs = ((rem as u64) << 32) | (base[j] as u64);
//             let rhs = 3u64;
//             let q = (lhs / rhs) as u32;
//             let r = (lhs % rhs) as u32;

//             base[j] = q;
//             rem = r;
//         }
//         trits[i] = rem as i8 - 1;
//     }

//     if flip_trits {
//         for v in trits.iter_mut() {
//             *v = -*v;
//         }
//     }
// }
