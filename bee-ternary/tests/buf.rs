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

fn create_generic<T: raw::RawEncodingBuf>() {
    assert!(TritBuf::<T>::new().len() == 0);
    fuzz(100, || {
        let len = thread_rng().gen_range(0, 100);
        assert!(TritBuf::<T>::zeros(len).len() == len);
    });
    fuzz(100, || {
        let trits = gen_buf::<T>(0..1000).1;
        assert!(TritBuf::<T>::from_i8_unchecked(&trits).len() == trits.len());
    });
}

fn eq_generic<T: raw::RawEncodingBuf + Clone>() {
    fuzz(100, || {
        let a = gen_buf::<T>(0..1000).0;
        let b = a.clone();

        assert_eq!(a, b);
    });
}

fn encode_generic<T: raw::RawEncodingBuf + Clone, U: raw::RawEncodingBuf>() {
    fuzz(100, || {
        let a = gen_buf::<T>(0..1000).0;
        let b = a.clone().into_encoding::<U>();

        assert_eq!(a, b);
        assert_eq!(a.len(), b.len());

        let c = b.into_encoding::<T>();

        assert_eq!(a, c);
        assert_eq!(a.len(), c.len());
    });
}

#[test]
fn create() {
    create_generic::<T1B1Buf>();
    create_generic::<T4B1Buf>();
}

#[test]
fn eq() {
    eq_generic::<T1B1Buf>();
    eq_generic::<T4B1Buf>();
}

#[test]
fn encode() {
    encode_generic::<T1B1Buf, T4B1Buf>();
    encode_generic::<T4B1Buf, T1B1Buf>();
    encode_generic::<T1B1Buf, T1B1Buf>();
    encode_generic::<T4B1Buf, T4B1Buf>();
}
