use bee_bundle::TransactionField;
use bee_ternary::{T1B1Buf, TritBuf, Trits, T1B1};

use rand::Rng;

pub fn rand_trits_field<T: TransactionField>() -> T
// Bit weird, but these constraints permit generating random trits for any transaction field type
where
    T::Inner: ToOwned<Owned = TritBuf>,
    TritBuf: std::borrow::Borrow<T::Inner>,
{
    const TRIT_SET: &[i8] = &[-1, 0, 1];
    let mut rng = rand::thread_rng();

    let raw_buffer: Vec<i8> = (0..T::trit_len())
        .map(|_| {
            let idx = rng.gen_range(0, TRIT_SET.len());
            TRIT_SET[idx]
        })
        .collect();

    let trits = Trits::<T1B1>::try_from_raw(raw_buffer.as_slice(), T::trit_len())
        .unwrap()
        .to_buf::<T1B1Buf>();
    T::from_inner_unchecked(trits)
}
