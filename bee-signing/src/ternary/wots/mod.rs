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

mod shake;
mod sponge;

pub use shake::{WotsShakePrivateKeyGenerator, WotsShakePrivateKeyGeneratorBuilder};
pub use sponge::{WotsSpongePrivateKeyGenerator, WotsSpongePrivateKeyGeneratorBuilder};

use crate::ternary::{PrivateKey, PublicKey, RecoverableSignature, Signature};

use bee_common_derive::{SecretDebug, SecretDisplay, SecretDrop};
use bee_crypto_ext::ternary::sponge::Sponge;
use bee_ternary::{TritBuf, Trits};

use thiserror::Error;
use zeroize::Zeroize;

use std::{
    convert::TryFrom,
    fmt::{self, Display, Formatter},
    marker::PhantomData,
};

/// Errors occuring during WOTS operations.
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    /// Invalid security level provided.
    #[error("Invalid security level provided.")]
    InvalidSecurityLevel,
    /// Missing security level.
    #[error("Missing security level.")]
    MissingSecurityLevel,
    /// Failed sponge operation.
    #[error("Failed sponge operation.")]
    FailedSpongeOperation,
}

/// Available WOTS security levels.
#[derive(Clone, Copy)]
pub enum WotsSecurityLevel {
    /// Low security.
    Low = 1,
    /// Medium security.
    Medium = 2,
    /// High security.
    High = 3,
}

impl Default for WotsSecurityLevel {
    fn default() -> Self {
        WotsSecurityLevel::Medium
    }
}

impl TryFrom<u8> for WotsSecurityLevel {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(WotsSecurityLevel::Low),
            2 => Ok(WotsSecurityLevel::Medium),
            3 => Ok(WotsSecurityLevel::High),
            _ => Err(Error::InvalidSecurityLevel),
        }
    }
}

/// A Winternitz One Time Signature private key.
#[derive(SecretDebug, SecretDisplay, SecretDrop)]
pub struct WotsPrivateKey<S> {
    pub(crate) state: TritBuf,
    pub(crate) _sponge: PhantomData<S>,
}

impl<S> Zeroize for WotsPrivateKey<S> {
    fn zeroize(&mut self) {
        // This unsafe is fine since we only reset the whole buffer with zeros, there is no alignement issues.
        unsafe { self.state.as_i8_slice_mut().zeroize() }
    }
}

impl<S: Sponge + Default> PrivateKey for WotsPrivateKey<S> {
    type PublicKey = WotsPublicKey<S>;
    type Signature = WotsSignature<S>;
    type Error = Error;

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

impl<S: Sponge + Default> WotsPrivateKey<S> {
    /// Returns the inner trits.
    pub fn as_trits(&self) -> &Trits {
        &self.state
    }
}

/// A Winternitz One Time Signature public key.
pub struct WotsPublicKey<S> {
    state: TritBuf,
    _sponge: PhantomData<S>,
}

impl<S: Sponge + Default> PublicKey for WotsPublicKey<S> {
    type Signature = WotsSignature<S>;
    type Error = Error;

    // TODO: enforce hash size ?
    fn verify(&self, message: &[i8], signature: &Self::Signature) -> Result<bool, Self::Error> {
        Ok(signature.recover_public_key(message)?.state == self.state)
    }

    fn size(&self) -> usize {
        self.state.len()
    }

    fn from_trits(state: TritBuf) -> Self {
        Self {
            state,
            _sponge: PhantomData,
        }
    }

    fn to_trits(&self) -> &Trits {
        &self.state
    }
}

impl<S: Sponge + Default> Display for WotsPublicKey<S> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", trits_to_string(self.to_trits()))
    }
}

/// A Winternitz One Time Signature signature.
pub struct WotsSignature<S> {
    state: TritBuf,
    _sponge: PhantomData<S>,
}

impl<S: Sponge + Default> Signature for WotsSignature<S> {
    fn size(&self) -> usize {
        self.state.len()
    }

    fn from_trits(state: TritBuf) -> Self {
        Self {
            state,
            _sponge: PhantomData,
        }
    }

    fn to_trits(&self) -> &Trits {
        &self.state
    }
}

impl<S: Sponge + Default> RecoverableSignature for WotsSignature<S> {
    type PublicKey = WotsPublicKey<S>;
    type Error = Error;

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

impl<S: Sponge + Default> Display for WotsSignature<S> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", trits_to_string(self.to_trits()))
    }
}

// TODO consider making this a ternary utility function
fn trits_to_string(trits: &Trits) -> String {
    trits.iter_trytes().map(char::from).collect::<String>()
}
