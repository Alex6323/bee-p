pub mod constants;

use common::constants::{HASH_TRIT_LEN, NUM_HASHING_ROUNDS};
use common::Trit;

use constants::STATE_TRIT_LEN;
use constants::TRUTH_TABLE;

pub struct Curl {
    num_rounds: usize,
    state: [Trit; STATE_TRIT_LEN],
    scratchpad: [Trit; STATE_TRIT_LEN],
}

impl Curl {
    pub fn new(num_rounds: usize) -> Self {
        Self {
            num_rounds,
            ..Self::default()
        }
    }

    pub fn absorb(&mut self, trits: &[i8], mut offset: usize, mut length: usize) {
        loop {
            let chunk_length = {
                if length < HASH_TRIT_LEN {
                    length
                } else {
                    HASH_TRIT_LEN
                }
            };

            self.state[0..chunk_length].copy_from_slice(&trits[offset..offset + chunk_length]);

            self.transform();

            offset += chunk_length;

            if length > chunk_length {
                length -= chunk_length;
            } else {
                break;
            }
        }
    }

    pub fn squeeze(&mut self, trits: &mut [i8], mut offset: usize, mut length: usize) {
        loop {
            let chunk_length = {
                if length < HASH_TRIT_LEN {
                    length
                } else {
                    HASH_TRIT_LEN
                }
            };

            trits[offset..offset + chunk_length].copy_from_slice(&self.state[0..chunk_length]);

            self.transform();

            offset += chunk_length;

            if length > chunk_length {
                length -= chunk_length;
            } else {
                break;
            }
        }
    }

    pub fn reset(&mut self) {
        self.state.iter_mut().for_each(|t| *t = 0);
    }

    fn transform(&mut self) {
        let mut scratchpad_index = 0;

        for _ in 0..self.num_rounds {
            self.scratchpad.copy_from_slice(&self.state);
            for state_index in 0..STATE_TRIT_LEN {
                let prev_scratchpad_index = scratchpad_index;

                if scratchpad_index < 365 {
                    scratchpad_index += 364;
                } else {
                    scratchpad_index -= 365;
                }

                self.state[state_index] = TRUTH_TABLE[(self.scratchpad[prev_scratchpad_index]
                    + (self.scratchpad[scratchpad_index] << 2)
                    + 5) as usize];
            }
        }
    }
}

impl Default for Curl {
    fn default() -> Self {
        Curl {
            num_rounds: NUM_HASHING_ROUNDS,
            state: [0; STATE_TRIT_LEN],
            scratchpad: [0; STATE_TRIT_LEN],
        }
    }
}
