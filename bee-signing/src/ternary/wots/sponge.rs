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

use crate::ternary::{PrivateKeyGenerator, TernarySeed, WotsError, WotsPrivateKey, WotsSecurityLevel};

use bee_crypto::ternary::Sponge;
use bee_ternary::{TritBuf, Trits};

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

    fn generate_from_entropy(&self, entropy: &Trits) -> Result<Self::PrivateKey, Self::Error> {
        let mut sponge = S::default();
        let mut state = TritBuf::zeros(self.security_level as usize * 6561);

        sponge
            .digest_into(entropy, &mut state)
            .map_err(|_| Self::Error::FailedSpongeOperation)?;

        Ok(Self::PrivateKey {
            state,
            _sponge: PhantomData,
        })
    }
}
