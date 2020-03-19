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
            println!("{:?}", a);
            println!("{:?}", b);
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

fn encode_generic<T: raw::RawEncodingBuf + Clone, U: raw::RawEncodingBuf>() {
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
    create_generic::<T1B1Buf<BTrit>>();
    create_generic::<T1B1Buf<UTrit>>();
    create_generic::<T2B1Buf>();
    create_generic::<T3B1Buf>();
    create_generic::<T4B1Buf>();
}

#[test]
fn push_pop() {
    push_pop_generic::<T1B1Buf<BTrit>>();
    push_pop_generic::<T1B1Buf<UTrit>>();
    push_pop_generic::<T2B1Buf>();
    push_pop_generic::<T3B1Buf>();
    push_pop_generic::<T4B1Buf>();
}

#[test]
fn eq() {
    eq_generic::<T1B1Buf<BTrit>>();
    eq_generic::<T1B1Buf<UTrit>>();
    eq_generic::<T2B1Buf>();
    eq_generic::<T3B1Buf>();
    eq_generic::<T4B1Buf>();
}

#[test]
fn encode() {
    encode_generic::<T1B1Buf<BTrit>, T2B1Buf>();
    encode_generic::<T1B1Buf<UTrit>, T2B1Buf>();
    encode_generic::<T1B1Buf<BTrit>, T3B1Buf>();
    encode_generic::<T1B1Buf<UTrit>, T3B1Buf>();
    encode_generic::<T1B1Buf<BTrit>, T4B1Buf>();
    encode_generic::<T1B1Buf<UTrit>, T4B1Buf>();
    encode_generic::<T2B1Buf, T1B1Buf<BTrit>>();
    encode_generic::<T2B1Buf, T1B1Buf<UTrit>>();
    encode_generic::<T3B1Buf, T1B1Buf<BTrit>>();
    encode_generic::<T3B1Buf, T1B1Buf<UTrit>>();
    encode_generic::<T4B1Buf, T1B1Buf<BTrit>>();
    encode_generic::<T4B1Buf, T1B1Buf<UTrit>>();
    encode_generic::<T2B1Buf, T3B1Buf>();
    encode_generic::<T3B1Buf, T4B1Buf>();
    encode_generic::<T3B1Buf, T2B1Buf>();
    encode_generic::<T2B1Buf, T3B1Buf>();
    encode_generic::<T4B1Buf, T2B1Buf>();
    encode_generic::<T4B1Buf, T3B1Buf>();
}
