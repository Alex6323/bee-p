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

fn get_generic<T: raw::RawEncodingBuf + Clone>() {
    fuzz(100, || {
        let (a, a_i8) = gen_buf::<T>(1..1000);

        fuzz(25, || {
            assert_eq!(a.get(a.len() + thread_rng().gen_range(0, 20)), None);
        });

        let mut sl = a.as_slice();
        let mut sl_i8 = &a_i8[..];
        for _ in 0..20 {
            if sl.len() == 0 {
                break;
            }
            let i = thread_rng().gen_range(0, sl.len());
            assert_eq!(
                sl.get(i),
                Some(Trit::from(sl_i8[i])),
            );

            let idx = thread_rng().gen_range(0, sl.len());
            let len = thread_rng().gen_range(0, sl.len() - idx);
            sl_i8 = &sl_i8[idx..idx + len];
            sl = &sl[idx..idx + len];
        }
    });
}

fn set_generic<T: raw::RawEncodingBuf + Clone>() {
    fuzz(100, || {
        let (mut a, mut a_i8) = gen_buf::<T>(1..1000);

        fuzz(100, || {
            let mut sl = a.as_slice_mut();
            let mut sl_i8 = &mut a_i8[..];
            for _ in 0..10 {
                if sl.len() == 0 {
                    break;
                }

                let i = thread_rng().gen_range(0, sl.len());
                let trit = thread_rng().gen_range(-1i8, 2);

                sl.set(i, Trit::from(trit));
                sl_i8[i] = trit;

                assert_eq!(
                    sl.get(i),
                    Some(Trit::from(sl_i8[i])),
                );

                let idx = thread_rng().gen_range(0, sl.len());
                let len = thread_rng().gen_range(0, sl.len() - idx);
                sl_i8 = &mut sl_i8[idx..idx + len];
                sl = &mut sl[idx..idx + len];
            }

            assert!(a
                .iter()
                .zip(a_i8.iter())
                .all(|(a, b)| a == Trit::from(*b)));

            assert_eq!(a.len(), a_i8.len());
        });
    });
}

fn chunks_generic<T: raw::RawEncodingBuf + Clone>() {
    fuzz(100, || {
        let (a, a_i8) = gen_buf::<T>(2..1000);

        let chunk_len = thread_rng().gen_range(1, a.len());
        for (a, a_i8) in a.chunks(chunk_len).zip(a_i8.chunks(chunk_len)) {
            assert_eq!(a.len(), a_i8.len());
            assert!(a
                .iter()
                .zip(a_i8.iter())
                .all(|(a, b)| a == Trit::from(*b)));
        }
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
    get_generic::<T2B1Buf>();
    get_generic::<T3B1Buf>();
    get_generic::<T4B1Buf>();
}

#[test]
fn set() {
    set_generic::<T1B1Buf>();
    set_generic::<T2B1Buf>();
    set_generic::<T3B1Buf>();
    set_generic::<T4B1Buf>();
}

#[test]
#[should_panic]
fn set_panic() {
    set_panic_generic::<T1B1Buf>();
    set_panic_generic::<T2B1Buf>();
    set_panic_generic::<T3B1Buf>();
    set_panic_generic::<T4B1Buf>();
}

#[test]
fn chunks() {
    chunks_generic::<T1B1Buf>();
    chunks_generic::<T2B1Buf>();
    chunks_generic::<T3B1Buf>();
    chunks_generic::<T4B1Buf>();
}

#[test]
fn chunks_mut() {
    fuzz(100, || {
        let (mut a, mut a_i8) = gen_buf::<T1B1Buf>(2..1000);

        let chunk_len = thread_rng().gen_range(1, a.len());
        for (a, a_i8) in a.chunks_mut(chunk_len).zip(a_i8.chunks_mut(chunk_len)) {
            assert_eq!(a.len(), a_i8.len());
            assert!(a
                .iter()
                .zip(a_i8.iter())
                .all(|(a, b)| a == Trit::from(*b)));
        }
    });
}
