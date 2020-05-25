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

use crate::{IotaSeed, PrivateKey, PrivateKeyGenerator, PublicKey, RecoverableSignature, Seed, Signature};

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

#[derive(Default)]
pub struct WotsPrivateKeyGeneratorBuilder<S> {
    security_level: Option<WotsSecurityLevel>,
    _sponge: PhantomData<S>,
}

#[derive(Default)]
pub struct WotsPrivateKeyGenerator<S> {
    security_level: WotsSecurityLevel,
    _sponge: PhantomData<S>,
}

pub struct WotsPrivateKey<S> {
    state: TritBuf,
    _sponge: PhantomData<S>,
}

pub struct WotsPublicKey<S> {
    state: TritBuf,
    _sponge: PhantomData<S>,
}

pub struct WotsSignature<S> {
    state: TritBuf,
    _sponge: PhantomData<S>,
}

// TODO: documentation
#[derive(Debug, PartialEq)]
pub enum WotsError {
    MissingSecurityLevel,
    FailedSpongeOperation,
}

impl<S: Sponge + Default> WotsPrivateKeyGeneratorBuilder<S> {
    pub fn security_level(mut self, security_level: WotsSecurityLevel) -> Self {
        self.security_level.replace(security_level);
        self
    }

    pub fn build(self) -> Result<WotsPrivateKeyGenerator<S>, WotsError> {
        Ok(WotsPrivateKeyGenerator {
            security_level: self.security_level.ok_or(WotsError::MissingSecurityLevel)?,
            _sponge: PhantomData,
        })
    }
}

impl<S: Sponge + Default> PrivateKeyGenerator for WotsPrivateKeyGenerator<S> {
    type Seed = IotaSeed<S>;
    type PrivateKey = WotsPrivateKey<S>;
    type Error = WotsError;

    fn generate(&self, seed: &Self::Seed, index: u64) -> Result<Self::PrivateKey, Self::Error> {
        let subseed = seed.subseed(index);
        let mut sponge = S::default();
        let mut state = TritBuf::zeros(self.security_level as usize * 6561);

        sponge
            .digest_into(subseed.trits(), &mut state)
            .map_err(|_| Self::Error::FailedSpongeOperation)?;

        Ok(Self::PrivateKey {
            state,
            _sponge: PhantomData,
        })
    }
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

#[cfg(test)]
mod tests {

    use super::*;

    use bee_crypto::{CurlP27, CurlP81, Kerl};
    use bee_ternary::{T1B1Buf, TryteBuf};

    const SEED: &str = "NNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNN";
    const MESSAGE: &str = "CHXHLHQLOPYP9NSUXTMWWABIBSBLUFXFRNWOZXJPVJPBCIDI99YBSCFYILCHPXHTSEYSYWIGQFERCRVDD";

    #[test]
    fn wots_generator_missing_security_level() {
        match WotsPrivateKeyGeneratorBuilder::<Kerl>::default().build() {
            Ok(_) => unreachable!(),
            Err(err) => assert_eq!(err, WotsError::MissingSecurityLevel),
        }
    }

    #[test]
    fn wots_generator_valid() {
        let security_levels = vec![
            WotsSecurityLevel::Low,
            WotsSecurityLevel::Medium,
            WotsSecurityLevel::High,
        ];
        for security in security_levels {
            assert_eq!(
                WotsPrivateKeyGeneratorBuilder::<Kerl>::default()
                    .security_level(security)
                    .build()
                    .is_ok(),
                true
            );
        }
    }

    fn wots_generic_complete<S: Sponge + Default>() {
        let seed_trits = TryteBuf::try_from_str(SEED).unwrap().as_trits().encode::<T1B1Buf>();
        let message_trits = TryteBuf::try_from_str(MESSAGE).unwrap().as_trits().encode::<T1B1Buf>();
        let seed = IotaSeed::<S>::from_buf(seed_trits).unwrap();
        let security_levels = vec![
            WotsSecurityLevel::Low,
            WotsSecurityLevel::Medium,
            WotsSecurityLevel::High,
        ];
        for security in security_levels {
            for index in 0..5 {
                let private_key_generator = WotsPrivateKeyGeneratorBuilder::<S>::default()
                    .security_level(security)
                    .build()
                    .unwrap();
                let mut private_key = private_key_generator.generate(&seed, index).unwrap();
                let public_key = private_key.generate_public_key().unwrap();
                let signature = private_key.sign(message_trits.as_i8_slice()).unwrap();
                let recovered_public_key = signature.recover_public_key(message_trits.as_i8_slice()).unwrap();
                assert_eq!(public_key.as_bytes(), recovered_public_key.as_bytes());
                let valid = public_key.verify(message_trits.as_i8_slice(), &signature).unwrap();
                assert!(valid);
            }
        }
    }

    #[test]
    fn wots_kerl_complete() {
        wots_generic_complete::<Kerl>();
    }

    #[test]
    fn wots_curl27_complete() {
        wots_generic_complete::<CurlP27>();
    }

    #[test]
    fn wots_curl81_complete() {
        wots_generic_complete::<CurlP81>();
    }
}
