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

use crate::constants::{HASH_LEN, NUM_ROUNDS, STATE_LEN};

use bee_common::constants::NONCE_TRIT_LEN as NONCE_LEN;

use crate::{
    constants::*, cores::Cores, difficulty::Difficulty, input::InputTrits, nonce::NonceTrits,
    powcurlstate::PowCurlState,
};

use std::sync::{Arc, RwLock};

type Exhausted = bool;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum PearlDiverState {
    Created,
    Searching,
    Cancelled,
    Completed(Option<NonceTrits>),
}

#[derive(Clone)]
pub struct PearlDiver {
    cores: Cores,
    difficulty: Difficulty,
    state: Arc<RwLock<PearlDiverState>>,
}

impl PearlDiver {
    pub fn new(cores: Cores, difficulty: Difficulty) -> Self {
        Self {
            cores,
            difficulty,
            state: Arc::new(RwLock::new(PearlDiverState::Created)),
        }
    }

    pub fn search_sync(&mut self, input: &InputTrits) {
        assert!(self.state() == PearlDiverState::Created);

        let mut prestate = make_prestate(input);

        self.set_state(PearlDiverState::Searching);

        let num_cores = self.cores.clone();

        crossbeam::scope(|scope| {
            for _ in 0..*num_cores {
                let mut state_thr = prestate.clone();

                let pdstate = self.state.clone();
                let difficulty = self.difficulty.clone();

                scope.spawn(move |_| {
                    let mut state_tmp = PowCurlState::new(BITS_1);

                    while *pdstate.read().unwrap() == PearlDiverState::Searching {
                        unsafe {
                            transform(&mut state_thr, &mut state_tmp);
                        }

                        if let Some(nonce) = find_nonce(&state_thr, &difficulty) {
                            *pdstate.write().unwrap() = PearlDiverState::Completed(Some(nonce));
                            break;
                        } else {
                            if { inner_increment(&mut state_thr) } {
                                break;
                            }
                        }
                    }
                });

                outer_increment(&mut prestate);
            }
        })
        .unwrap();

        // If we reach this point, but the PearlDiver state hasn't been changed to `Completed` by some thread,
        // then we must have searched the whole space without finding a valid nonce, and we have to switch the
        // state to `Completed(None)` manually.
        if self.state() == PearlDiverState::Searching {
            self.set_state(PearlDiverState::Completed(None));
        }
    }

    pub fn cancel(&mut self) {
        if self.state() == PearlDiverState::Searching {
            self.set_state(PearlDiverState::Cancelled);
        }
    }

    pub fn state(&self) -> PearlDiverState {
        *self.state.read().unwrap()
    }

    pub fn set_state(&mut self, state: PearlDiverState) {
        *self.state.write().unwrap() = state;
    }
}

fn outer_increment(prestate: &mut PowCurlState) {
    for i in OUTER_INCR_START..INNER_INCR_START {
        let with_carry = prestate.bit_add(i);
        if !with_carry {
            break;
        }
    }
}

fn inner_increment(prestate: &mut PowCurlState) -> Exhausted {
    // we have not exhausted the search space until each add
    // operation produces a carry
    for i in INNER_INCR_START..HASH_LEN {
        if {
            let with_carry = prestate.bit_add(i);
            !with_carry
        } {
            return false;
        }
    }
    true
}

fn make_prestate(input: &InputTrits) -> PowCurlState {
    let mut prestate = PowCurlState::new(BITS_1);
    let mut tmpstate = PowCurlState::new(BITS_1);

    let mut offset = 0;

    for _ in 0..NUM_FULL_CHUNKS_FOR_PRESTATE {
        for i in 0..HASH_LEN {
            match (*input)[offset] {
                1 => prestate.set(i, BITS_1, BITS_0),
                -1 => prestate.set(i, BITS_0, BITS_1),
                _ => (),
            }
            offset += 1;
        }

        unsafe {
            transform(&mut prestate, &mut tmpstate);
        }
    }

    for i in 0..CHUNK_NONCE_START {
        match (*input)[offset] {
            1 => prestate.set(i, BITS_1, BITS_0),
            -1 => prestate.set(i, BITS_0, BITS_1),
            _ => (),
        }
        offset += 1;
    }

    prestate.set(CHUNK_NONCE_START, H0, L0);
    prestate.set(CHUNK_NONCE_START + 1, H1, L1);
    prestate.set(CHUNK_NONCE_START + 2, H2, L2);
    prestate.set(CHUNK_NONCE_START + 3, H3, L3);

    prestate
}

/// NOTE: To prevent unnecessary allocations we instantiate the scratchpad (tmp) only once per core outside of
/// this function.
unsafe fn transform(pre: &mut PowCurlState, tmp: &mut PowCurlState) {
    let (mut hpre, mut lpre) = pre.as_mut_ptr();
    let (mut htmp, mut ltmp) = tmp.as_mut_ptr();

    let mut lswp: *mut u64;
    let mut hswp: *mut u64;

    for _ in 0..(NUM_ROUNDS - 1) {
        for j in 0..STATE_LEN {
            let index1 = INDICES[j];
            let index2 = INDICES[j + 1];

            let alpha = *lpre.offset(index1);
            let kappa = *hpre.offset(index1);
            let sigma = *lpre.offset(index2);
            let gamma = *hpre.offset(index2);

            let delta = (alpha | !gamma) & (sigma ^ kappa);

            *ltmp.offset(j as isize) = !delta;
            *htmp.offset(j as isize) = (alpha ^ gamma) | delta;
        }

        lswp = lpre;
        hswp = hpre;
        lpre = ltmp;
        hpre = htmp;
        ltmp = lswp;
        htmp = hswp;
    }

    // NOTE: Since we don't compute a new state after that, we stop after 'HASH_LEN'.
    for j in 0..HASH_LEN {
        let index1 = INDICES[j];
        let index2 = INDICES[j + 1];

        let alpha = *lpre.offset(index1);
        let kappa = *hpre.offset(index1);
        let sigma = *lpre.offset(index2);
        let gamma = *hpre.offset(index2);

        let delta = (alpha | !gamma) & (sigma ^ kappa);

        *lpre.offset(j as isize) = !delta;
        *hpre.offset(j as isize) = (alpha ^ gamma) | delta;
    }
}

fn find_nonce(state: &PowCurlState, difficulty: &Difficulty) -> Option<NonceTrits> {
    let mut nonce_test = BITS_1;

    for i in (HASH_LEN - **difficulty)..HASH_LEN {
        nonce_test &= state.bit_equal(i);

        // If 'nonce_test' ever becomes 0, then this means that none of the current nonce candidates satisfied
        // the difficulty setting
        if nonce_test == 0 {
            return None;
        }
    }

    for slot in 0..BATCH_SIZE {
        if (nonce_test >> slot) & 1 != 0 {
            return Some(extract_nonce(&state, slot));
        }
    }

    unreachable!()
}

/// Extracts the nonce from the final Curl state and the given slot index.
fn extract_nonce(state: &PowCurlState, slot: usize) -> NonceTrits {
    let mut nonce = [0; NONCE_LEN];
    let slotmask = 1 << slot;

    for (offset, i) in (CHUNK_NONCE_START..HASH_LEN).enumerate() {
        let (hi, lo) = state.get(i);

        match (hi & slotmask, lo & slotmask) {
            (1, 0) => nonce[offset] = 1,
            (0, 1) => nonce[offset] = -1,
            (_, _) => (),
        }
    }

    NonceTrits(nonce)
}

impl Default for PearlDiver {
    fn default() -> Self {
        Self::new(Cores::max(), Difficulty::mainnet())
    }
}
