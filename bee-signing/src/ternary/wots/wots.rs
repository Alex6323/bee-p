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

use crate::ternary::{PrivateKey, PublicKey, RecoverableSignature, Signature};

use bee_crypto::Sponge;
use bee_ternary::{TritBuf, Trits};

use std::marker::PhantomData;

#[derive(Clone, Copy)]
pub enum WotsSecurityLevel {
    Low = 1,
    Medium = 2,
    High = 3,
}

impl Default for WotsSecurityLevel {
    fn default() -> Self {
        WotsSecurityLevel::Medium
    }
}

#[derive(Debug, PartialEq)]
pub enum WotsError {
    MissingSecurityLevel,
    FailedSpongeOperation,
}

pub struct WotsPrivateKey<S> {
    pub(crate) state: TritBuf,
    pub(crate) _sponge: PhantomData<S>,
}

impl<S: Sponge + Default> PrivateKey for WotsPrivateKey<S> {
    type PublicKey = WotsPublicKey<S>;
    type Signature = WotsSignature<S>;
    type Error = WotsError;

    fn generate_public_key(&self) -> Result<Self::PublicKey, Self::Error> {
        let mut sponge = S::default();
        let mut hashed_private_key = self.state.clone();
        let mut digests: TritBuf = TritBuf::zeros((self.state.len() / 6561) * 243);
        let mut hash = TritBuf::zeros(243);

        for chunk in hashed_private_key.chunks_mut(243) {
            for _ in 0..26 {
                sponge.absorb(chunk).map_err(|_| Self::Error::FailedSpongeOperation)?;
                sponge
                    .squeeze_into(chunk)
                    .map_err(|_| Self::Error::FailedSpongeOperation)?;
                sponge.reset();
            }
        }

        for (i, chunk) in hashed_private_key.chunks(6561).enumerate() {
            sponge
                .digest_into(chunk, &mut digests[i * 243..(i + 1) * 243])
                .map_err(|_| Self::Error::FailedSpongeOperation)?;
        }

        sponge
            .digest_into(&digests, &mut hash)
            .map_err(|_| Self::Error::FailedSpongeOperation)?;

        Ok(Self::PublicKey {
            state: hash,
            _sponge: PhantomData,
        })
    }

    // TODO: enforce hash size ?
    fn sign(&mut self, message: &[i8]) -> Result<Self::Signature, Self::Error> {
        let mut sponge = S::default();
        let mut signature = self.state.clone();

        for (i, chunk) in signature.chunks_mut(243).enumerate() {
            let val = message[i * 3] + message[i * 3 + 1] * 3 + message[i * 3 + 2] * 9;

            for _ in 0..(13 - val) {
                sponge.absorb(chunk).map_err(|_| Self::Error::FailedSpongeOperation)?;
                sponge
                    .squeeze_into(chunk)
                    .map_err(|_| Self::Error::FailedSpongeOperation)?;
                sponge.reset();
            }
        }

        Ok(Self::Signature {
            state: signature,
            _sponge: PhantomData,
        })
    }
}

pub struct WotsPublicKey<S> {
    state: TritBuf,
    _sponge: PhantomData<S>,
}

impl<S: Sponge + Default> PublicKey for WotsPublicKey<S> {
    type Signature = WotsSignature<S>;
    type Error = WotsError;

    // TODO: enforce hash size ?
    fn verify(&self, message: &[i8], signature: &Self::Signature) -> Result<bool, Self::Error> {
        Ok(signature.recover_public_key(message)?.state == self.state)
    }

    fn from_buf(state: TritBuf) -> Self {
        Self {
            state,
            _sponge: PhantomData,
        }
    }

    fn as_bytes(&self) -> &[i8] {
        self.state.as_i8_slice()
    }

    fn trits(&self) -> &Trits {
        &self.state
    }
}

pub struct WotsSignature<S> {
    state: TritBuf,
    _sponge: PhantomData<S>,
}

// TODO default impl ?
impl<S: Sponge + Default> Signature for WotsSignature<S> {
    fn size(&self) -> usize {
        self.state.len()
    }

    fn from_buf(state: TritBuf) -> Self {
        Self {
            state,
            _sponge: PhantomData,
        }
    }

    fn as_bytes(&self) -> &[i8] {
        self.state.as_i8_slice()
    }

    fn trits(&self) -> &Trits {
        &self.state
    }
}

impl<S: Sponge + Default> RecoverableSignature for WotsSignature<S> {
    type PublicKey = WotsPublicKey<S>;
    type Error = WotsError;

    fn recover_public_key(&self, message: &[i8]) -> Result<Self::PublicKey, Self::Error> {
        let mut sponge = S::default();
        let mut hash = TritBuf::zeros(243);
        let mut digests: TritBuf = TritBuf::zeros((self.state.len() / 6561) * 243);
        let mut state = self.state.clone();

        for (i, chunk) in state.chunks_mut(243).enumerate() {
            let val = message[i * 3] + message[i * 3 + 1] * 3 + message[i * 3 + 2] * 9;

            for _ in 0..(val - -13) {
                sponge.absorb(chunk).map_err(|_| Self::Error::FailedSpongeOperation)?;
                sponge
                    .squeeze_into(chunk)
                    .map_err(|_| Self::Error::FailedSpongeOperation)?;
                sponge.reset();
            }
        }

        for (i, chunk) in state.chunks(6561).enumerate() {
            sponge
                .digest_into(chunk, &mut digests[i * 243..(i + 1) * 243])
                .map_err(|_| Self::Error::FailedSpongeOperation)?;
        }

        sponge
            .digest_into(&digests, &mut hash)
            .map_err(|_| Self::Error::FailedSpongeOperation)?;

        Ok(Self::PublicKey {
            state: hash,
            _sponge: PhantomData,
        })
    }
}
