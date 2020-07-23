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

//! Merkle Signature Scheme.

use crate::ternary::{PrivateKey, PrivateKeyGenerator, PublicKey, RecoverableSignature, Seed, Signature};

use bee_common_derive::{SecretDebug, SecretDisplay, SecretDrop};
use bee_crypto::ternary::sponge::Sponge;
use bee_ternary::{T1B1Buf, TritBuf, Trits, T1B1};

use thiserror::Error;
use zeroize::Zeroize;

use std::marker::PhantomData;

/// Errors occuring during MSS operations.
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    /// Invalid MSS depth provided.
    #[error("Invalid MSS depth provided.")]
    InvalidDepth(u8),
    /// Missing MSS depth.
    #[error("Missing MSS depth.")]
    MissingDepth,
    /// Missing underlying private key generator.
    #[error("Missing underlying private key generator.")]
    MissingGenerator,
    /// Underlying private key generation failed.
    #[error("Underlying private key generation failed.")]
    FailedUnderlyingPrivateKeyGeneration,
    /// Underlying ppublic key generation failed.
    #[error("Underlying ppublic key generation failed.")]
    FailedUnderlyingPublicKeyGeneration,
    /// Underlying signature generation failed.
    #[error("Underlying signature generation failed.")]
    FailedUnderlyingSignatureGeneration,
    /// Underlying public key recovery failed.
    #[error("Underlying public key recovery failed.")]
    FailedUnderlyingPublicKeyRecovery,
    /// Failed sponge operation.
    #[error("Failed sponge operation.")]
    FailedSpongeOperation,
    /// Seed generation failed.
    #[error("Seed generation failed.")]
    FailedSeed,
}

/// Merkle Signature Scheme private key generator builder.
pub struct MssPrivateKeyGeneratorBuilder<S, G> {
    depth: Option<u8>,
    generator: Option<G>,
    _sponge: PhantomData<S>,
}

impl<S, G> Default for MssPrivateKeyGeneratorBuilder<S, G>
where
    S: Sponge + Default,
    G: PrivateKeyGenerator,
{
    fn default() -> Self {
        Self {
            depth: None,
            generator: None,
            _sponge: PhantomData,
        }
    }
}

impl<S, G> MssPrivateKeyGeneratorBuilder<S, G>
where
    S: Sponge + Default,
    G: PrivateKeyGenerator,
{
    /// Sets the depth of the MSS.
    pub fn depth(mut self, depth: u8) -> Self {
        self.depth = Some(depth);
        self
    }

    /// Sets the underlying private key generator.
    pub fn generator(mut self, generator: G) -> Self {
        self.generator = Some(generator);
        self
    }

    /// Builds the private key generator.
    pub fn build(self) -> Result<MssPrivateKeyGenerator<S, G>, Error> {
        let depth = match self.depth {
            Some(depth) => match depth {
                0..=20 => depth,
                _ => return Err(Error::InvalidDepth(depth)),
            },
            None => return Err(Error::MissingDepth),
        };
        let generator = self.generator.ok_or(Error::MissingGenerator)?;

        Ok(MssPrivateKeyGenerator {
            depth,
            generator,
            _sponge: PhantomData,
        })
    }
}

/// Merkle Signature Scheme private key generator.
pub struct MssPrivateKeyGenerator<S, G> {
    depth: u8,
    generator: G,
    _sponge: PhantomData<S>,
}

impl<S, G> PrivateKeyGenerator for MssPrivateKeyGenerator<S, G>
where
    S: Sponge + Default,
    G: PrivateKeyGenerator,
    <<<G as PrivateKeyGenerator>::PrivateKey as PrivateKey>::PublicKey as PublicKey>::Signature: RecoverableSignature,
{
    type Seed = G::Seed;
    type PrivateKey = MssPrivateKey<S, G::PrivateKey>;
    type Error = Error;

    fn generate_from_entropy(&self, entropy: &Trits<T1B1>) -> Result<Self::PrivateKey, Self::Error> {
        let seed = Self::Seed::from_trits(entropy.to_buf()).map_err(|_| Error::FailedSeed)?;
        let mut sponge = S::default();
        let mut keys = Vec::new();
        let mut tree = TritBuf::<T1B1Buf>::zeros(((1 << self.depth) - 1) * 243);

        // TODO: reserve ?

        for key_index in 0..(1 << (self.depth - 1)) {
            let ots_private_key = self
                .generator
                .generate_from_entropy(seed.subseed(key_index).as_trits())
                .map_err(|_| Self::Error::FailedUnderlyingPrivateKeyGeneration)?;
            let ots_public_key = ots_private_key
                .generate_public_key()
                .map_err(|_| Self::Error::FailedUnderlyingPublicKeyGeneration)?;
            let tree_index = ((1 << (self.depth - 1)) + key_index - 1) as usize;

            keys.push(ots_private_key);
            tree[tree_index * 243..(tree_index + 1) * 243].copy_from(ots_public_key.as_trits());
        }

        for depth in (0..self.depth - 1).rev() {
            for i in 0..(1 << depth) {
                let index = (1 << depth) + i - 1;
                let left_index = index * 2 + 1;
                let right_index = left_index + 1;

                sponge
                    .absorb(&tree[left_index * 243..(left_index + 1) * 243])
                    .map_err(|_| Self::Error::FailedSpongeOperation)?;
                sponge
                    .absorb(&tree[right_index * 243..(right_index + 1) * 243])
                    .map_err(|_| Self::Error::FailedSpongeOperation)?;
                sponge
                    .squeeze_into(&mut tree[index * 243..(index + 1) * 243])
                    .map_err(|_| Self::Error::FailedSpongeOperation)?;
                sponge.reset();
            }
        }

        Ok(MssPrivateKey {
            depth: self.depth,
            index: 0,
            keys,
            tree,
            _sponge: PhantomData,
        })
    }
}

/// Merkle Signature Scheme private key.
#[derive(SecretDebug, SecretDisplay, SecretDrop)]
pub struct MssPrivateKey<S, K: Zeroize> {
    depth: u8,
    index: u64,
    keys: Vec<K>,
    tree: TritBuf<T1B1Buf>,
    _sponge: PhantomData<S>,
}

impl<S, K: Zeroize> Zeroize for MssPrivateKey<S, K> {
    fn zeroize(&mut self) {
        for key in self.keys.iter_mut() {
            key.zeroize();
        }
        unsafe { self.tree.as_i8_slice_mut().zeroize() }
    }
}

impl<S, K> PrivateKey for MssPrivateKey<S, K>
where
    S: Sponge + Default,
    K: PrivateKey,
    <<K as PrivateKey>::PublicKey as PublicKey>::Signature: RecoverableSignature,
{
    type PublicKey = MssPublicKey<S, K::PublicKey>;
    type Signature = MssSignature<S>;
    type Error = Error;

    fn generate_public_key(&self) -> Result<Self::PublicKey, Self::Error> {
        // TODO return or generate ?
        Ok(Self::PublicKey::from_trits(self.tree[0..243].to_buf()).depth(self.depth))
    }

    fn sign(&mut self, message: &Trits<T1B1>) -> Result<Self::Signature, Self::Error> {
        let ots_private_key = &mut self.keys[self.index as usize];
        let ots_signature = ots_private_key
            .sign(message)
            .map_err(|_| Self::Error::FailedUnderlyingSignatureGeneration)?;
        // let mut state = vec![0; ots_signature.size() + 6561];
        let mut state = TritBuf::<T1B1Buf>::zeros(ots_signature.size() + 6561);
        let mut tree_index = ((1 << (self.depth - 1)) + self.index - 1) as usize;
        let mut sibling_index;
        let mut i = 0;

        // TODO PAD TO 6561
        state[0..ots_signature.size()].copy_from(ots_signature.as_trits());

        while tree_index != 0 {
            if tree_index % 2 != 0 {
                sibling_index = tree_index + 1;
                tree_index /= 2;
            } else {
                sibling_index = tree_index - 1;
                tree_index = (tree_index - 1) / 2;
            }

            state[ots_signature.size() + i * 243..ots_signature.size() + (i + 1) * 243]
                .copy_from(&self.tree[sibling_index * 243..(sibling_index + 1) * 243]);
            i += 1;
        }

        self.index += 1;

        Ok(Self::Signature::from_trits(state).index(self.index - 1))
    }
}

/// Merkle Signature Scheme public key.
pub struct MssPublicKey<S, K> {
    state: TritBuf<T1B1Buf>,
    depth: u8,
    _sponge: PhantomData<S>,
    _key: PhantomData<K>,
}

impl<S, K> MssPublicKey<S, K>
where
    S: Sponge + Default,
    K: PublicKey,
{
    /// Sets the depth of the public key.
    pub fn depth(mut self, depth: u8) -> Self {
        self.depth = depth;
        self
    }
}

impl<S, K> PublicKey for MssPublicKey<S, K>
where
    S: Sponge + Default,
    K: PublicKey,
    <K as PublicKey>::Signature: RecoverableSignature,
{
    type Signature = MssSignature<S>;
    type Error = Error;

    fn verify(&self, message: &Trits<T1B1>, signature: &Self::Signature) -> Result<bool, Self::Error> {
        let mut sponge = S::default();
        let ots_signature =
            K::Signature::from_trits(signature.state[0..((signature.state.len() / 6561) - 1) * 6561].to_buf());
        let siblings: TritBuf<T1B1Buf> = signature.state.chunks(6561).last().unwrap().to_buf();
        let ots_public_key = ots_signature
            .recover_public_key(message)
            .map_err(|_| Self::Error::FailedUnderlyingPublicKeyRecovery)?;
        let mut hash = TritBuf::<T1B1Buf>::zeros(243);

        hash.copy_from(ots_public_key.as_trits());

        let mut j = 1;
        for (i, sibling) in siblings.chunks(243).enumerate() {
            if self.depth - 1 == i as u8 {
                break;
            }

            if signature.index & j != 0 {
                sponge.absorb(sibling).map_err(|_| Self::Error::FailedSpongeOperation)?;
                sponge.absorb(&hash).map_err(|_| Self::Error::FailedSpongeOperation)?;
            } else {
                sponge.absorb(&hash).map_err(|_| Self::Error::FailedSpongeOperation)?;
                sponge
                    .absorb(&sibling)
                    .map_err(|_| Self::Error::FailedSpongeOperation)?;
            }
            sponge
                .squeeze_into(&mut hash)
                .map_err(|_| Self::Error::FailedSpongeOperation)?;
            sponge.reset();

            j <<= 1;
        }

        Ok(hash == self.state)
    }

    fn size(&self) -> usize {
        self.state.len()
    }

    fn from_trits(state: TritBuf<T1B1Buf>) -> Self {
        Self {
            state,
            // TODO OPTION
            depth: 0,
            _sponge: PhantomData,
            _key: PhantomData,
        }
    }

    fn as_trits(&self) -> &Trits<T1B1> {
        &self.state
    }
}

/// Merkle Signature Scheme signature.
pub struct MssSignature<S> {
    state: TritBuf<T1B1Buf>,
    index: u64,
    _sponge: PhantomData<S>,
}

impl<S: Sponge + Default> MssSignature<S> {
    /// Set the index of the signature.
    pub fn index(mut self, index: u64) -> Self {
        self.index = index;
        self
    }
}

// TODO default impl ?
impl<S: Sponge + Default> Signature for MssSignature<S> {
    fn size(&self) -> usize {
        self.state.len()
    }

    fn from_trits(state: TritBuf<T1B1Buf>) -> Self {
        Self {
            state,
            // TODO OPTION
            index: 0,
            _sponge: PhantomData,
        }
    }

    fn as_trits(&self) -> &Trits<T1B1> {
        &self.state
    }
}
