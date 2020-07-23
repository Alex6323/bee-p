// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use bee_ternary::{T1B1Buf, T3B1Buf, TritBuf, Trits, Tryte, T1B1, T3B1};

const SECURITY_LEVEL_MAX: usize = 3;
const NORMALIZED_FRAGMENT_LENGTH: usize = 27;

// TODO Trits or TritBuf ?
pub fn normalize_hash(hash: &Trits<T1B1>) -> TritBuf<T1B1Buf> {
    let hash_trits = hash.as_i8_slice();
    let mut normalized_hash = [0i8; SECURITY_LEVEL_MAX * NORMALIZED_FRAGMENT_LENGTH];

    for i in 0..SECURITY_LEVEL_MAX {
        let mut sum: i16 = 0;

        for j in (i * NORMALIZED_FRAGMENT_LENGTH)..((i + 1) * NORMALIZED_FRAGMENT_LENGTH) {
            normalized_hash[j] = hash_trits[j * 3] + hash_trits[j * 3 + 1] * 3 + hash_trits[j * 3 + 2] * 9;
            sum += normalized_hash[j] as i16;
        }

        if sum > 0 {
            while sum > 0 {
                for j in (i * NORMALIZED_FRAGMENT_LENGTH)..((i + 1) * NORMALIZED_FRAGMENT_LENGTH) {
                    if (normalized_hash[j] as i8) > Tryte::MIN_VALUE as i8 {
                        normalized_hash[j] -= 1;
                        break;
                    }
                }
                sum -= 1;
            }
        } else {
            while sum < 0 {
                for j in (i * NORMALIZED_FRAGMENT_LENGTH)..((i + 1) * NORMALIZED_FRAGMENT_LENGTH) {
                    if (normalized_hash[j] as i8) < Tryte::MAX_VALUE as i8 {
                        normalized_hash[j] += 1;
                        break;
                    }
                }
                sum += 1;
            }
        }
    }

    unsafe {
        Trits::<T3B1>::from_raw_unchecked(&normalized_hash, normalized_hash.len() * 3)
            .to_buf::<T3B1Buf>()
            .encode::<T1B1Buf>()
    }
}
