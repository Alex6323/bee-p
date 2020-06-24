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

use crate::ternary::{Sponge, HASH_LEN};

use bee_ternary::{Btrit, TritBuf, Trits};

use std::{
    convert::{Infallible, TryInto},
    ops::{Deref, DerefMut},
};

/// The length internal state of the `CurlP` sponge construction (in units of binary-coded, balanced trits).
const STATE_LEN: usize = HASH_LEN * 3;
const HALF_STATE_LEN: usize = STATE_LEN / 2;
const TRUTH_TABLE: [i8; 11] = [1, 0, -1, 2, 1, -1, 0, 2, -1, 1, 0];

#[derive(Copy, Clone)]
pub enum CurlPRounds {
    Rounds27 = 27,
    Rounds81 = 81,
}

pub struct CurlP {
    /// The number of rounds of hashing to apply before a hash is squeezed.
    rounds: CurlPRounds,
    /// The internal state.
    state: TritBuf,
    /// Workspace for performing transformations
    work_state: TritBuf,
}

impl CurlP {
    /// Create a new `CurlP` sponge with `rounds` of iterations.
    pub fn new(rounds: CurlPRounds) -> Self {
        Self {
            rounds,
            state: TritBuf::zeros(STATE_LEN),
            work_state: TritBuf::zeros(STATE_LEN),
        }
    }

    /// Transforms the internal state of the `CurlP` sponge after the input was copied into the internal state.
    ///
    /// The essence of this transformation is the application of a so-called substitution box to the internal state,
    /// which happens `round` number of times.
    fn transform(&mut self) {
        fn calculate_truth_table_index(xs: &Trits, p: usize, q: usize) -> usize {
            let idx = xs.get(p).unwrap() as i8 + ((xs.get(q).unwrap() as i8) << 2) + 5;
            idx as usize
        }

        fn apply_substitution_box(input: &Trits, output: &mut Trits) {
            assert!(input.len() <= STATE_LEN);
            assert!(output.len() <= STATE_LEN);

            // Unwrapping here and below is acceptable because we have verified that `calculate_truth_table_index` and
            // `TRUTH_TABLE` always yield a value in {-1, 0, 1}
            output.set(
                0,
                TRUTH_TABLE[calculate_truth_table_index(input, 0, HALF_STATE_LEN)]
                    .try_into()
                    .unwrap(),
            );

            for state_index in 0..HALF_STATE_LEN {
                let left_idx = HALF_STATE_LEN - state_index;
                let right_idx = STATE_LEN - state_index - 1;

                output.set(
                    2 * state_index + 1,
                    TRUTH_TABLE[calculate_truth_table_index(input, left_idx, right_idx)]
                        .try_into()
                        .unwrap(),
                );

                let left_idx = left_idx - 1;
                output.set(
                    2 * state_index + 2,
                    TRUTH_TABLE[calculate_truth_table_index(input, right_idx, left_idx)]
                        .try_into()
                        .unwrap(),
                );
            }
        }

        let (lhs, rhs) = (&mut self.state, &mut self.work_state);

        for _ in 0..self.rounds as usize {
            apply_substitution_box(&lhs, rhs);
            std::mem::swap(lhs, rhs);
        }

        // Swap the slices back if the number of rounds is even (otherwise `self.work_state` contains the transformed
        // state).
        if self.rounds as usize & 1 == 0 {
            std::mem::swap(lhs, rhs);
        }
    }
}

impl Sponge for CurlP {
    type Error = Infallible;

    /// Absorb `input` into the sponge by copying `HASH_LEN` chunks of it into its internal state and transforming the
    /// state before moving on to the next chunk.
    ///
    /// If `input` is not a multiple of `HASH_LEN` with the last chunk having `n < HASH_LEN` trits, the last chunk will
    /// be copied to the first `n` slots of the internal state. The remaining data in the internal state is then just
    /// the result of the last transformation before the data was copied, and will be reused for the next
    /// transformation.
    fn absorb(&mut self, input: &Trits) -> Result<(), Self::Error> {
        for chunk in input.chunks(HASH_LEN) {
            self.state[0..chunk.len()].copy_from(chunk);
            self.transform();
        }
        Ok(())
    }

    /// Reset the internal state by overwriting it with zeros.
    fn reset(&mut self) {
        self.state.fill(Btrit::Zero);
    }

    /// Squeeze the sponge by copying the calculated hash into the provided `buf`. This will fill the buffer in chunks
    /// of `HASH_LEN` at a time.
    ///
    /// If the last chunk is smaller than `HASH_LEN`, then only the fraction that fits is written into it.
    fn squeeze_into(&mut self, buf: &mut Trits) -> Result<(), Self::Error> {
        for chunk in buf.chunks_mut(HASH_LEN) {
            chunk.copy_from(&self.state[0..chunk.len()]);
            self.transform()
        }
        Ok(())
    }
}

/// `CurlP` with a fixed number of 27 rounds.
pub struct CurlP27(CurlP);

impl CurlP27 {
    pub fn new() -> Self {
        Self(CurlP::new(CurlPRounds::Rounds27))
    }
}

impl Default for CurlP27 {
    fn default() -> Self {
        CurlP27::new()
    }
}

impl Deref for CurlP27 {
    type Target = CurlP;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CurlP27 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// `CurlP` with a fixed number of 81 rounds.
pub struct CurlP81(CurlP);

impl CurlP81 {
    pub fn new() -> Self {
        Self(CurlP::new(CurlPRounds::Rounds81))
    }
}

impl Default for CurlP81 {
    fn default() -> Self {
        CurlP81::new()
    }
}

impl Deref for CurlP81 {
    type Target = CurlP;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CurlP81 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}