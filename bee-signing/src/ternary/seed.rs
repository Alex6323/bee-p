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

use bee_common_derive::{SecretDebug, SecretDisplay, SecretDrop};
use bee_crypto::ternary::sponge::Sponge;
use bee_ternary::{Btrit, T1B1Buf, Trit, TritBuf, Trits, T1B1};

use rand::Rng;
use thiserror::Error;
use zeroize::Zeroize;

use std::marker::PhantomData;

/// Ternary `Seed` to derive private keys, public keys and signatures from.
pub trait Seed: Zeroize + Drop {
    /// Associated error type.
    type Error;

    /// Creates a new `Seed`.
    fn new() -> Self;

    /// Creates a new `Seed` from the current `Seed` and an index.
    fn subseed(&self, index: u64) -> Self;

    /// Creates a `Seed` from trits.
    fn from_trits(buf: TritBuf<T1B1Buf>) -> Result<Self, Self::Error>
    where
        Self: Sized;

    /// Returns the inner trits.
    fn as_trits(&self) -> &Trits<T1B1>;
}

/// Errors occuring when handling a `Seed`.
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    /// Invalid seed length.
    #[error("Invalid seed length.")]
    InvalidLength(usize),
    /// Failed sponge operation.
    #[error("Failed sponge operation.")]
    FailedSpongeOperation,
}

/// Ternary `Sponge`-based `Seed` to derive private keys, public keys and signatures from.
#[derive(SecretDebug, SecretDisplay, SecretDrop)]
pub struct TernarySeed<S> {
    seed: TritBuf<T1B1Buf>,
    _sponge: PhantomData<S>,
}

impl<S> Zeroize for TernarySeed<S> {
    fn zeroize(&mut self) {
        unsafe { self.seed.as_i8_slice_mut().zeroize() }
    }
}

impl<S: Sponge + Default> Seed for TernarySeed<S> {
    type Error = Error;

    fn new() -> Self {
        // `ThreadRng` implements `CryptoRng` so it is safe to use in cryptographic contexts.
        // https://rust-random.github.io/rand/rand/trait.CryptoRng.html
        let mut rng = rand::thread_rng();
        // TODO out of here ?
        let trits = [-1, 0, 1];
        let seed: Vec<i8> = (0..243).map(|_| trits[rng.gen_range(0, trits.len())]).collect();

        Self {
            seed: unsafe { Trits::<T1B1>::from_raw_unchecked(&seed, 243).to_buf() },
            _sponge: PhantomData,
        }
    }

    fn subseed(&self, index: u64) -> Self {
        let mut sponge = S::default();
        let mut subseed = self.seed.clone();

        for _ in 0..index {
            for t in subseed.iter_mut() {
                if let Some(ntrit) = t.checked_increment() {
                    *t = ntrit;
                    break;
                } else {
                    *t = Btrit::NegOne;
                }
            }
        }

        // TODO return error
        let tmp = match sponge.digest(&subseed) {
            Ok(buf) => buf,
            Err(_) => unreachable!(),
        };

        Self {
            seed: tmp,
            _sponge: PhantomData,
        }
    }

    fn from_trits(buf: TritBuf<T1B1Buf>) -> Result<Self, Self::Error> {
        if buf.len() != 243 {
            return Err(Self::Error::InvalidLength(buf.len()));
        }

        Ok(Self {
            seed: buf,
            _sponge: PhantomData,
        })
    }

    fn as_trits(&self) -> &Trits<T1B1> {
        &self.seed
    }
}
