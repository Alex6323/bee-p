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

use crate::ternary::{
    wots::{Error as WotsError, WotsPrivateKey, WotsSecurityLevel},
    PrivateKeyGenerator, TernarySeed,
};

use bee_crypto::ternary::bigint::{binary_representation::U8Repr, endianness::BigEndian, I384, T242, T243};
use bee_crypto_ext::ternary::sponge::Sponge;
use bee_ternary::{Btrit, T1B1Buf, TritBuf, Trits};

use sha3::{
    digest::{ExtendableOutput, Update, XofReader},
    Shake256,
};

use std::marker::PhantomData;

/// Shake-based Winternitz One Time Signature private key generator builder.
#[derive(Default)]
pub struct WotsShakePrivateKeyGeneratorBuilder<S> {
    security_level: Option<WotsSecurityLevel>,
    _sponge: PhantomData<S>,
}

impl<S: Sponge + Default> WotsShakePrivateKeyGeneratorBuilder<S> {
    /// Sets the security level of the private key.
    pub fn security_level(mut self, security_level: WotsSecurityLevel) -> Self {
        self.security_level.replace(security_level);
        self
    }

    /// Builds the private key generator.
    pub fn build(self) -> Result<WotsShakePrivateKeyGenerator<S>, WotsError> {
        Ok(WotsShakePrivateKeyGenerator {
            security_level: self.security_level.ok_or(WotsError::MissingSecurityLevel)?,
            _sponge: PhantomData,
        })
    }
}

/// Shake-based Winternitz One Time Signature private key generator.
pub struct WotsShakePrivateKeyGenerator<S> {
    security_level: WotsSecurityLevel,
    _sponge: PhantomData<S>,
}

impl<S: Sponge + Default> PrivateKeyGenerator for WotsShakePrivateKeyGenerator<S> {
    type Seed = TernarySeed<S>;
    type PrivateKey = WotsPrivateKey<S>;
    type Error = WotsError;

    fn generate_from_entropy(&self, entropy: &Trits) -> Result<Self::PrivateKey, Self::Error> {
        let mut state = TritBuf::<T1B1Buf>::zeros(self.security_level as usize * 6561);
        let mut shake = Shake256::default();
        let mut ternary_buffer = T243::<Btrit>::default();
        ternary_buffer.copy_from(entropy);
        let mut binary_buffer: I384<BigEndian, U8Repr> = ternary_buffer.into_t242().into();

        shake.update(&binary_buffer[..]);
        let mut reader = shake.finalize_xof();

        for trit_chunk in state.chunks_mut(243) {
            reader.read(&mut binary_buffer[..]);
            let ternary_value = T242::from_i384_ignoring_mst(binary_buffer).into_t243();

            trit_chunk.copy_from(&ternary_value);
        }

        Ok(Self::PrivateKey {
            state,
            _sponge: PhantomData,
        })
    }
}
