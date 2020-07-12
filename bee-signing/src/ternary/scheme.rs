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

use crate::ternary::Seed;

use bee_ternary::{TritBuf, Trits};

use zeroize::Zeroize;

pub trait PrivateKeyGenerator {
    type Seed: Seed;
    /// The type of the generated private keys
    type PrivateKey: PrivateKey;
    type Error;

    /// Deterministically generates and returns a private key from a seed
    ///
    /// # Arguments
    ///
    /// * `seed`    A seed to deterministically derive a private key from
    /// * `index`   An index to deterministically derive a private key from
    ///
    /// # Example
    ///
    /// ```
    /// use bee_crypto::ternary::Kerl;
    /// use bee_signing::ternary::{
    ///     PrivateKeyGenerator, Seed, TernarySeed, WotsSecurityLevel, WotsSpongePrivateKeyGeneratorBuilder,
    /// };
    ///
    /// let seed = TernarySeed::<Kerl>::new();
    /// let private_key_generator = WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default()
    ///     .security_level(WotsSecurityLevel::Medium)
    ///     .build()
    ///     .unwrap();
    /// let private_key = private_key_generator.generate_from_seed(&seed, 0);
    /// ```
    fn generate_from_seed(&self, seed: &Self::Seed, index: u64) -> Result<Self::PrivateKey, Self::Error> {
        self.generate_from_entropy(seed.subseed(index).trits())
    }

    /// Deterministically generates and returns a private key from entropy
    ///
    /// # Arguments
    ///
    /// * `entropy` Entropy to deterministically derive a private key from
    ///
    /// # Example
    ///
    /// ```
    /// use bee_crypto::ternary::Kerl;
    /// use bee_signing::ternary::{
    ///     PrivateKeyGenerator, Seed, TernarySeed, WotsSecurityLevel, WotsSpongePrivateKeyGeneratorBuilder,
    /// };
    ///
    /// let seed = TernarySeed::<Kerl>::new();
    /// let private_key_generator = WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default()
    ///     .security_level(WotsSecurityLevel::Medium)
    ///     .build()
    ///     .unwrap();
    /// let private_key = private_key_generator.generate_from_entropy(seed.trits());
    /// ```
    fn generate_from_entropy(&self, entropy: &Trits) -> Result<Self::PrivateKey, Self::Error>;
}

pub trait PrivateKey: Zeroize + Drop {
    /// The type of the matching public key
    type PublicKey: PublicKey;
    /// The type of the generated signatures
    type Signature: Signature;
    type Error;

    /// Returns the public counterpart of a private key
    ///
    /// # Example
    ///
    /// ```
    /// # use bee_crypto::ternary::Kerl;
    /// # use bee_signing::ternary::{
    ///     TernarySeed,
    ///     PrivateKeyGenerator,
    ///     Seed,
    ///     WotsSpongePrivateKeyGeneratorBuilder,
    ///     WotsSecurityLevel,
    /// };
    /// use bee_signing::ternary::PrivateKey;
    ///
    /// # let seed = TernarySeed::<Kerl>::new();
    /// # let private_key_generator = WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default()
    ///     .security_level(WotsSecurityLevel::Medium)
    ///     .build()
    ///     .unwrap();
    /// # let private_key = private_key_generator.generate_from_seed(&seed, 0).unwrap();
    /// let public_key = private_key.generate_public_key();
    /// ```
    fn generate_public_key(&self) -> Result<Self::PublicKey, Self::Error>;

    /// Generates and returns a signature for a given message
    ///
    /// # Arguments
    ///
    /// * `message` A slice that holds a message to be signed
    ///
    /// # Example
    ///
    /// ```
    /// # use bee_crypto::ternary::Kerl;
    /// # use bee_signing::ternary::{
    ///     TernarySeed,
    ///     PrivateKeyGenerator,
    ///     Seed,
    ///     WotsSpongePrivateKeyGeneratorBuilder,
    ///     WotsSecurityLevel,
    /// };
    /// use bee_signing::ternary::PrivateKey;
    /// use bee_ternary::{
    ///     T1B1Buf,
    ///     TryteBuf,
    /// };
    ///
    /// # let seed = TernarySeed::<Kerl>::new();
    /// # let private_key_generator = WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default()
    ///     .security_level(WotsSecurityLevel::Medium)
    ///     .build()
    ///     .unwrap();
    /// # let mut private_key = private_key_generator.generate_from_seed(&seed, 0).unwrap();
    /// let message = "CHXHLHQLOPYP9NSUXTMWWABIBSBLUFXFRNWOZXJPVJPBCIDI99YBSCFYILCHPXHTSEYSYWIGQFERCRVDD";
    /// let message_trits = TryteBuf::try_from_str(message).unwrap().as_trits().encode::<T1B1Buf>();
    /// let signature = private_key.sign(&message_trits.as_i8_slice());
    /// ```
    fn sign(&mut self, message: &[i8]) -> Result<Self::Signature, Self::Error>;
}

pub trait PublicKey {
    type Signature: Signature;
    type Error;

    fn verify(&self, message: &[i8], signature: &Self::Signature) -> Result<bool, Self::Error>;

    fn size(&self) -> usize;

    fn from_trits(buf: TritBuf) -> Self;

    fn to_trits(&self) -> &Trits;
}

pub trait Signature {
    fn size(&self) -> usize;

    fn from_trits(buf: TritBuf) -> Self;

    fn to_trits(&self) -> &Trits;
}

pub trait RecoverableSignature: Signature {
    type PublicKey: PublicKey;
    type Error;

    fn recover_public_key(&self, message: &[i8]) -> Result<Self::PublicKey, Self::Error>;
}
