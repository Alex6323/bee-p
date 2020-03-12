#![cfg(feature = "serde1")]

mod common;
use self::common::*;

use rand::prelude::*;
use bee_ternary::*;

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
    serialize_generic::<T1B1Buf<Btrit>>();
    serialize_generic::<T1B1Buf<Utrit>>();
    serialize_generic::<T2B1Buf>();
    serialize_generic::<T3B1Buf>();
    serialize_generic::<T4B1Buf>();
}

#[test]
fn deserialize() {
    deserialize_generic::<T1B1Buf<Btrit>>();
    deserialize_generic::<T1B1Buf<Utrit>>();
    deserialize_generic::<T2B1Buf>();
    deserialize_generic::<T3B1Buf>();
    deserialize_generic::<T4B1Buf>();
}
