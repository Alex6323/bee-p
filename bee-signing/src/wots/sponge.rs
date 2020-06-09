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

use crate::{PrivateKeyGenerator, Seed, TernarySeed, WotsError, WotsPrivateKey, WotsSecurityLevel};

use bee_crypto::Sponge;
use bee_ternary::TritBuf;

use std::marker::PhantomData;

#[derive(Default)]
pub struct WotsSpongePrivateKeyGeneratorBuilder<S> {
    security_level: Option<WotsSecurityLevel>,
    _sponge: PhantomData<S>,
}

impl<S: Sponge + Default> WotsSpongePrivateKeyGeneratorBuilder<S> {
    pub fn security_level(mut self, security_level: WotsSecurityLevel) -> Self {
        self.security_level.replace(security_level);
        self
    }

    pub fn build(self) -> Result<WotsSpongePrivateKeyGenerator<S>, WotsError> {
        Ok(WotsSpongePrivateKeyGenerator {
            security_level: self.security_level.ok_or(WotsError::MissingSecurityLevel)?,
            _sponge: PhantomData,
        })
    }
}

pub struct WotsSpongePrivateKeyGenerator<S> {
    security_level: WotsSecurityLevel,
    _sponge: PhantomData<S>,
}

impl<S: Sponge + Default> PrivateKeyGenerator for WotsSpongePrivateKeyGenerator<S> {
    type Seed = TernarySeed<S>;
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

#[cfg(test)]
mod tests {

    use super::*;

    use crate::{PrivateKey, PublicKey, RecoverableSignature};

    use bee_crypto::{CurlP27, CurlP81, Kerl};
    use bee_ternary::{T1B1Buf, TryteBuf};

    const SEED: &str = "NNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNN";
    const MESSAGE: &str = "CHXHLHQLOPYP9NSUXTMWWABIBSBLUFXFRNWOZXJPVJPBCIDI99YBSCFYILCHPXHTSEYSYWIGQFERCRVDD";

    #[test]
    fn wots_generator_missing_security_level() {
        match WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default().build() {
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
                WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default()
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
        let seed = TernarySeed::<S>::from_buf(seed_trits).unwrap();
        let security_levels = vec![
            WotsSecurityLevel::Low,
            WotsSecurityLevel::Medium,
            WotsSecurityLevel::High,
        ];
        for security in security_levels {
            for index in 0..5 {
                let private_key_generator = WotsSpongePrivateKeyGeneratorBuilder::<S>::default()
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
