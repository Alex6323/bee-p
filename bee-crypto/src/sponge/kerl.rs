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

use crate::Sponge;

use bee_ternary::{Btrit, Trits, T1B1};
use bee_ternary_ext::bigint::{
    common::{BigEndian, U8Repr},
    I384, T242, T243,
};

use tiny_keccak::{Hasher, Keccak};

const HASH_LEN: usize = 243;

#[derive(Clone)]
pub struct Kerl {
    keccak: Keccak,
    binary_buffer: I384<BigEndian, U8Repr>,
    ternary_buffer: T243<Btrit>,
}

impl Kerl {
    pub fn new() -> Self {
        Self {
            keccak: Keccak::v384(),
            binary_buffer: I384::<BigEndian, U8Repr>::default(),
            ternary_buffer: T243::<Btrit>::default(),
        }
    }
}

impl Default for Kerl {
    fn default() -> Self {
        Kerl::new()
    }
}

#[derive(Debug)]
pub enum Error {
    NotMultipleOfHashLength,
    TernaryBinaryConversion(bee_ternary_ext::bigint::common::Error),
}

impl From<bee_ternary_ext::bigint::common::Error> for Error {
    fn from(error: bee_ternary_ext::bigint::common::Error) -> Self {
        Error::TernaryBinaryConversion(error)
    }
}

impl Sponge for Kerl {
    const IN_LEN: usize = HASH_LEN;
    const OUT_LEN: usize = HASH_LEN;

    type Error = Error;

    /// Absorb `input` into the sponge by copying `HASH_LEN` chunks of it into its internal
    /// state and transforming the state before moving on to the next chunk.
    ///
    /// If `input` is not a multiple of `HASH_LEN` with the last chunk having `n < HASH_LEN` trits,
    /// the last chunk will be copied to the first `n` slots of the internal state. The remaining
    /// data in the internal state is then just the result of the last transformation before the
    /// data was copied, and will be reused for the next transformation.
    fn absorb(&mut self, input: &Trits) -> Result<(), Self::Error> {
        if input.len() % Self::IN_LEN != 0 {
            return Err(Error::NotMultipleOfHashLength);
        }

        for trits_chunk in input.chunks(Self::IN_LEN) {
            self.ternary_buffer.inner_mut().copy_from(&trits_chunk);
            // Unwrapping is ok because this cannot fail.
            //
            // TODO: Replace with a dedicated `TryFrom` implementation with `Error = !`.
            //
            // TODO: Convert to `t242` without cloning.
            //
            // TODO: Convert to binary without cloning.
            self.binary_buffer = self.ternary_buffer.clone().into_t242().into();

            self.keccak.update(self.binary_buffer.inner_ref());
        }

        Ok(())
    }

    /// Reset the internal state by overwriting it with zeros.
    fn reset(&mut self) {
        // TODO: Overwrite the internal buffer directly rather then setting it to a new Keccak
        // object. This requires using `KeccakState::reset` via a new method `Keccak::method`
        // calling its internal state.
        self.keccak = Keccak::v384();
    }

    /// Squeeze the sponge by copying the calculated hash into the provided `buf`. This will fill
    /// the buffer in chunks of `HASH_LEN` at a time.
    ///
    /// If the last chunk is smaller than `HASH_LEN`, then only the fraction that fits is written
    /// into it.
    fn squeeze_into(&mut self, buf: &mut Trits<T1B1>) -> Result<(), Self::Error> {
        if buf.len() % Self::OUT_LEN != 0 {
            return Err(Error::NotMultipleOfHashLength);
        }

        for trit_chunk in buf.chunks_mut(Self::OUT_LEN) {
            // Create a new Keccak in lieu of resetting the internal one
            let mut keccak = Keccak::v384();

            // Swap out the internal one and the new one
            std::mem::swap(&mut self.keccak, &mut keccak);

            keccak.finalize(&mut self.binary_buffer.inner_mut()[..]);
            let ternary_value = T242::from_i384_ignoring_mst(self.binary_buffer).into_t243();

            trit_chunk.copy_from(&ternary_value.inner_ref());
            self.binary_buffer.not_inplace();
            self.keccak.update(self.binary_buffer.inner_ref());
        }
        Ok(())
    }
}
