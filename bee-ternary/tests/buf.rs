mod common;
use self::common::*;

use rand::prelude::*;
use bee_ternary::*;

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

fn create_unbalanced<T: raw::RawEncodingBuf>() {
    assert!(TritBuf::<T>::new().len() == 0);
    fuzz(100, || {
        let len = thread_rng().gen_range(0, 100);
        assert!(TritBuf::<T>::zeros(len).len() == len);
    });
    fuzz(100, || {
        let trits = gen_buf_unbalanced::<T>(0..1000).1;
        assert!(TritBuf::<T>::from_i8_unchecked(&trits).unwrap().len() == trits.len());
    });
}

fn push_pop_generic<T: raw::RawEncodingBuf>() {
    fuzz(100, || {
        let (mut a, mut b) = gen_buf::<T>(0..100);

        for _ in 0..1000 {
            if thread_rng().gen() {
                let trit = gen_trit();
                a.push(trit.into());
                b.push(trit);
            } else {
                assert_eq!(a.pop(), b.pop().map(Into::into));
            }
            // println!("{:?}", a);
            // println!("{:?}", b);
        }
    });
}

fn push_pop_generic_unbalanced<T: raw::RawEncodingBuf>() {
    fuzz(100, || {
        let (mut a, mut b) = gen_buf_unbalanced::<T>(0..100);

        for _ in 0..1000 {
            if thread_rng().gen() {
                let trit = gen_trit() + 1;
                a.push(trit.into());
                b.push(trit);
            } else {
                assert_eq!(a.pop(), b.pop().map(Into::into));
            }
            // println!("{:?}", a);
            // println!("{:?}", b);
        }
    });
}

fn eq_generic<T: raw::RawEncodingBuf + Clone>() {
    fuzz(100, || {
        let a = gen_buf::<T>(0..1000).0;
        let b = a.clone();

        assert_eq!(a, b);
    });
}

fn eq_generic_unbalanced<T: raw::RawEncodingBuf + Clone>() {
    fuzz(100, || {
        let a = gen_buf_unbalanced::<T>(0..1000).0;
        let b = a.clone();

        assert_eq!(a, b);
    });
}

fn encode_generic<T: raw::RawEncodingBuf + Clone, U: raw::RawEncodingBuf>()
where
    U::Slice: raw::RawEncoding<Trit = <T::Slice as raw::RawEncoding>::Trit>,
{
    fuzz(100, || {
        let a = gen_buf::<T>(0..100).0;
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
    create_generic::<T1B1Buf<Btrit>>();
    create_unbalanced::<T1B1Buf<Utrit>>();
    create_generic::<T2B1Buf>();
    create_generic::<T3B1Buf>();
    create_generic::<T4B1Buf>();
    create_generic::<T5B1Buf>();
}

#[test]
fn push_pop() {
    push_pop_generic::<T1B1Buf<Btrit>>();
    push_pop_generic_unbalanced::<T1B1Buf<Utrit>>();
    push_pop_generic::<T2B1Buf>();
    push_pop_generic::<T3B1Buf>();
    push_pop_generic::<T4B1Buf>();
    push_pop_generic::<T5B1Buf>();
}

#[test]
fn eq() {
    eq_generic::<T1B1Buf<Btrit>>();
    eq_generic_unbalanced::<T1B1Buf<Utrit>>();
    eq_generic::<T2B1Buf>();
    eq_generic::<T3B1Buf>();
    eq_generic::<T4B1Buf>();
    eq_generic::<T5B1Buf>();
}

#[test]
fn encode() {
    encode_generic::<T1B1Buf<Btrit>, T2B1Buf>();
    // encode_generic::<T1B1Buf<Utrit>, T2B1Buf>();
    encode_generic::<T1B1Buf<Btrit>, T3B1Buf>();
    // encode_generic::<T1B1Buf<Utrit>, T3B1Buf>();
    encode_generic::<T1B1Buf<Btrit>, T4B1Buf>();
    // encode_generic::<T1B1Buf<Utrit>, T4B1Buf>();
    encode_generic::<T2B1Buf, T1B1Buf<Btrit>>();
    // encode_generic::<T2B1Buf, T1B1Buf<Utrit>>();
    encode_generic::<T3B1Buf, T1B1Buf<Btrit>>();
    // encode_generic::<T3B1Buf, T1B1Buf<Utrit>>();
    encode_generic::<T4B1Buf, T1B1Buf<Btrit>>();
    // encode_generic::<T4B1Buf, T1B1Buf<Utrit>>();
    encode_generic::<T5B1Buf, T1B1Buf<Btrit>>();
    // encode_generic::<T5B1Buf, T1B1Buf<Utrit>>();
    encode_generic::<T2B1Buf, T3B1Buf>();
    encode_generic::<T3B1Buf, T4B1Buf>();
    encode_generic::<T3B1Buf, T5B1Buf>();
    encode_generic::<T3B1Buf, T2B1Buf>();
    encode_generic::<T2B1Buf, T3B1Buf>();
    encode_generic::<T4B1Buf, T2B1Buf>();
    encode_generic::<T4B1Buf, T3B1Buf>();
    encode_generic::<T5B1Buf, T2B1Buf>();
    encode_generic::<T5B1Buf, T3B1Buf>();
}
