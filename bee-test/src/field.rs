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

use bee_ternary::{T1B1Buf, TritBuf, Trits, T1B1};
use bee_transaction::TransactionField;

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
