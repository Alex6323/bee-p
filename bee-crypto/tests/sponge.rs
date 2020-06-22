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

use bee_crypto::ternary::Sponge;
use bee_ternary::{T1B1Buf, T3B1Buf, TritBuf, TryteBuf};

pub fn sponge_generic_digest<S: Sponge + Default>(input: &str, output: &str) {
    let mut sponge = S::default();

    let input_trit_buf = TryteBuf::try_from_str(input).unwrap().as_trits().encode::<T1B1Buf>();
    let expected_hash = TryteBuf::try_from_str(output).unwrap();
    let calculated_hash = match sponge.digest(input_trit_buf.as_slice()) {
        Ok(digest) => digest.encode::<T3B1Buf>(),
        Err(_) => unreachable!(),
    };

    assert_eq!(calculated_hash.as_slice(), expected_hash.as_trits());
}

pub fn sponge_generic_digest_into<S: Sponge + Default>(input: &str, output: &str) {
    let mut sponge = S::default();

    let input_trit_buf = TryteBuf::try_from_str(input).unwrap().as_trits().encode::<T1B1Buf>();
    let expected_hash = TryteBuf::try_from_str(output).unwrap();

    let output_len = expected_hash.as_trits().len();
    let mut calculated_hash = TritBuf::<T1B1Buf>::zeros(output_len);

    assert!(sponge
        .digest_into(input_trit_buf.as_slice(), &mut calculated_hash.as_slice_mut())
        .is_ok());

    let calculated_hash = calculated_hash.encode::<T3B1Buf>();

    assert_eq!(calculated_hash.as_slice(), expected_hash.as_trits());
}
