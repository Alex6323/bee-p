use std::ops::Range;
use rand::prelude::*;
use ternary::*;

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

fn get_generic<T: raw::RawEncodingBuf + Clone>() {
    fuzz(100, || {
        let (a, a_i8) = gen_buf::<T>(1..1000);

        fuzz(25, || {
            assert_eq!(a.get(a.len() + thread_rng().gen_range(0, 20)), None);
        });

        fuzz(100, || {
            let i = thread_rng().gen_range(0, a.len());
            assert_eq!(
                a.get(i),
                Some(Trit::from(a_i8[i])),
            );
        });
    });
}

fn set_generic<T: raw::RawEncodingBuf + Clone>() {
    fuzz(100, || {
        let (mut a, mut a_i8) = gen_buf::<T>(1..1000);

        fuzz(100, || {
            let i = thread_rng().gen_range(0, a.len());
            let trit = thread_rng().gen_range(-1i8, 2);

            a.set(i, Trit::from(trit));
            a_i8[i] = trit;

            assert_eq!(
                a.get(i),
                Some(Trit::from(a_i8[i])),
            );

            assert!(a
                .iter()
                .zip(a_i8.iter())
                .all(|(a, b)| a == Trit::from(*b)));

            assert_eq!(a.len(), a_i8.len());
        });
    });
}

fn set_panic_generic<T: raw::RawEncodingBuf + Clone>() {
    let mut a = gen_buf::<T>(0..1000).0;
    let len = a.len();
    a.set(len, Trit::Zero);
}

#[test]
fn get() {
    get_generic::<T1B1Buf>();
    get_generic::<T4B1Buf>();
}

#[test]
fn set() {
    set_generic::<T1B1Buf>();
    set_generic::<T4B1Buf>();
}

#[test]
#[should_panic]
fn set_panic() {
    set_panic_generic::<T1B1Buf>();
    set_panic_generic::<T4B1Buf>();
}
