use bee_ternary::{
    T1B1Buf,
    T3B1Buf,
    TritBuf,
    Trits,
    MAX_TRYTE_VALUE,
    MIN_TRYTE_VALUE,
    T3B1,
};

const SECURITY_LEVEL_MAX: usize = 3;
const NORMALIZED_FRAGMENT_LENGTH: usize = 27;

// TODO Trits or TritBuf ?
pub fn normalize_hash(hash: &Trits) -> TritBuf {
    let hash_trits = hash.as_i8_slice();
    let mut normalized_hash = [0i8; SECURITY_LEVEL_MAX * NORMALIZED_FRAGMENT_LENGTH];

    for i in 0..SECURITY_LEVEL_MAX {
        let mut sum: i16 = 0;

        for j in (i * NORMALIZED_FRAGMENT_LENGTH)..((i + 1) * NORMALIZED_FRAGMENT_LENGTH) {
            normalized_hash[j] = hash_trits[j * 3] + hash_trits[j * 3 + 1] * 3 + hash_trits[j * 3 + 2] * 9;
            sum = sum + normalized_hash[j] as i16;
        }

        if sum > 0 {
            while sum > 0 {
                for j in (i * NORMALIZED_FRAGMENT_LENGTH)..((i + 1) * NORMALIZED_FRAGMENT_LENGTH) {
                    if (normalized_hash[j] as i8) > MIN_TRYTE_VALUE {
                        normalized_hash[j] = (normalized_hash[j] as i8) - 1;
                        break;
                    }
                }
                sum = sum - 1;
            }
        } else {
            while sum < 0 {
                for j in (i * NORMALIZED_FRAGMENT_LENGTH)..((i + 1) * NORMALIZED_FRAGMENT_LENGTH) {
                    if (normalized_hash[j] as i8) < MAX_TRYTE_VALUE {
                        normalized_hash[j] = normalized_hash[j] + 1;
                        break;
                    }
                }
                sum = sum + 1;
            }
        }
    }

    unsafe {
        Trits::<T3B1>::from_raw_unchecked(&normalized_hash, normalized_hash.len() * 3)
            .to_buf::<T3B1Buf>()
            .encode::<T1B1Buf>()
    }
}
