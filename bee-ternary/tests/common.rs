use std::ops::Range;
use rand::prelude::*;
use bee_ternary::*;

pub fn gen_trit() -> i8 {
    (thread_rng().gen::<u8>() % 3) as i8 - 1
}

pub fn gen_buf<T: raw::RawEncodingBuf>(len: Range<usize>) -> (TritBuf<T>, Vec<i8>) {
    let len = thread_rng().gen_range(len.start, len.end);
    let trits = (0..len)
        .map(|_| gen_trit())
        .collect::<Vec<_>>();
    (TritBuf::<T>::from_i8_unchecked(&trits).unwrap(), trits)
}

// Not exactly fuzzing, just doing something a lot
pub fn fuzz(n: usize, mut f: impl FnMut()) {
    (0..n).for_each(|_| f());
}
