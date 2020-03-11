#![cfg(feature = "serde1")]

use std::ops::Range;
use rand::prelude::*;
use bee_ternary::*;

fn gen_buf<T: raw::RawEncodingBuf>(len: Range<usize>) -> (TritBuf<T>, Vec<i8>) {
    let len = thread_rng().gen_range(len.start, len.end);
    let trits = (0..len)
        .map(|_| (thread_rng().gen::<u8>() % 3) as i8 - 1)
        .collect::<Vec<_>>();
    (TritBuf::<T>::from_i8_unchecked(&trits), trits)
}

// Not exactly fuzzing, just doing something a lot
fn fuzz(n: usize, mut f: impl FnMut()) {
    (0..n).for_each(|_| f());
}

fn serialize_generic<T: raw::RawEncodingBuf>() {
    let (a, a_i8) = gen_buf::<T>(0..1000);
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        format!("[{}]", a_i8.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(",")),
    );
}

fn deserialize_generic<T: raw::RawEncodingBuf>() {
    let (a, a_i8) = gen_buf::<T>(0..1000);
    assert_eq!(
        serde_json::from_str::<TritBuf<T>>(&format!("[{}]", a_i8.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(","))).unwrap(),
        a,
    );
}

#[test]
fn serialize() {
    serialize_generic::<T1B1Buf<BTrit>>();
    serialize_generic::<T1B1Buf<UTrit>>();
    serialize_generic::<T2B1Buf>();
    serialize_generic::<T3B1Buf>();
    serialize_generic::<T4B1Buf>();
}

#[test]
fn deserialize() {
    deserialize_generic::<T1B1Buf<BTrit>>();
    deserialize_generic::<T1B1Buf<UTrit>>();
    deserialize_generic::<T2B1Buf>();
    deserialize_generic::<T3B1Buf>();
    deserialize_generic::<T4B1Buf>();
}
