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

use crate::ternary::{PrivateKey, PrivateKeyGenerator, PublicKey, RecoverableSignature, Seed, Signature};

use bee_common_derive::{SecretDebug, SecretDisplay, SecretDrop};
use bee_crypto_ext::ternary::Sponge;
use bee_ternary::{TritBuf, Trits};

use zeroize::Zeroize;

use std::marker::PhantomData;

#[derive(Debug, PartialEq)]
pub enum MssError {
    InvalidDepth(u8),
    MissingDepth,
    MissingGenerator,
    FailedUnderlyingPrivateKeyGeneration,
    FailedUnderlyingPublicKeyGeneration,
    FailedUnderlyingSignatureGeneration,
    FailedUnderlyingPublicKeyRecovery,
    FailedSpongeOperation,
    FailedSeed,
}

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
    pub fn depth(mut self, depth: u8) -> Self {
        self.depth = Some(depth);
        self
    }

    pub fn generator(mut self, generator: G) -> Self {
        self.generator = Some(generator);
        self
    }

    pub fn build(self) -> Result<MssPrivateKeyGenerator<S, G>, MssError> {
        let depth = match self.depth {
            Some(depth) => match depth {
                0..=20 => depth,
                _ => return Err(MssError::InvalidDepth(depth)),
            },
            None => return Err(MssError::MissingDepth),
        };
        let generator = self.generator.ok_or(MssError::MissingGenerator)?;

        Ok(MssPrivateKeyGenerator {
            depth,
            generator,
            _sponge: PhantomData,
        })
    }
}

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
    type Error = MssError;

    fn generate_from_entropy(&self, entropy: &Trits) -> Result<Self::PrivateKey, Self::Error> {
        let seed = Self::Seed::from_trits(entropy.to_buf()).map_err(|_| MssError::FailedSeed)?;
        let mut sponge = S::default();
        let mut keys = Vec::new();
        let mut tree = TritBuf::zeros(((1 << self.depth) - 1) * 243);

        // TODO: reserve ?

        for key_index in 0..(1 << (self.depth - 1)) {
            let ots_private_key = self
                .generator
                .generate_from_entropy(seed.subseed(key_index).to_trits())
                .map_err(|_| Self::Error::FailedUnderlyingPrivateKeyGeneration)?;
            let ots_public_key = ots_private_key
                .generate_public_key()
                .map_err(|_| Self::Error::FailedUnderlyingPublicKeyGeneration)?;
            let tree_index = ((1 << (self.depth - 1)) + key_index - 1) as usize;

            keys.push(ots_private_key);
            tree[tree_index * 243..(tree_index + 1) * 243].copy_from(ots_public_key.to_trits());
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

#[derive(SecretDebug, SecretDisplay, SecretDrop)]
pub struct MssPrivateKey<S, K: Zeroize> {
    depth: u8,
    index: u64,
    keys: Vec<K>,
    tree: TritBuf,
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
    type Error = MssError;

    fn generate_public_key(&self) -> Result<Self::PublicKey, Self::Error> {
        // TODO return or generate ?
        Ok(Self::PublicKey::from_trits(self.tree[0..243].to_buf()).depth(self.depth))
    }

    fn sign(&mut self, message: &[i8]) -> Result<Self::Signature, Self::Error> {
        let ots_private_key = &mut self.keys[self.index as usize];
        let ots_signature = ots_private_key
            .sign(message)
            .map_err(|_| Self::Error::FailedUnderlyingSignatureGeneration)?;
        // let mut state = vec![0; ots_signature.size() + 6561];
        let mut state: TritBuf = TritBuf::zeros(ots_signature.size() + 6561);
        let mut tree_index = ((1 << (self.depth - 1)) + self.index - 1) as usize;
        let mut sibling_index;
        let mut i = 0;

        // TODO PAD TO 6561
        state[0..ots_signature.size()].copy_from(ots_signature.to_trits());

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

pub struct MssPublicKey<S, K> {
    state: TritBuf,
    depth: u8,
    _sponge: PhantomData<S>,
    _key: PhantomData<K>,
}

impl<S, K> MssPublicKey<S, K>
where
    S: Sponge + Default,
    K: PublicKey,
{
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
    type Error = MssError;

    fn verify(&self, message: &[i8], signature: &Self::Signature) -> Result<bool, Self::Error> {
        let mut sponge = S::default();
        let ots_signature =
            K::Signature::from_trits(signature.state[0..((signature.state.len() / 6561) - 1) * 6561].to_buf());
        let siblings: TritBuf = signature.state.chunks(6561).last().unwrap().to_buf();
        let ots_public_key = ots_signature
            .recover_public_key(message)
            .map_err(|_| Self::Error::FailedUnderlyingPublicKeyRecovery)?;
        let mut hash: TritBuf = TritBuf::zeros(243);

        hash.copy_from(ots_public_key.to_trits());

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

    fn from_trits(state: TritBuf) -> Self {
        Self {
            state,
            // TODO OPTION
            depth: 0,
            _sponge: PhantomData,
            _key: PhantomData,
        }
    }

    fn to_trits(&self) -> &Trits {
        &self.state
    }
}

pub struct MssSignature<S> {
    state: TritBuf,
    index: u64,
    _sponge: PhantomData<S>,
}

impl<S: Sponge + Default> MssSignature<S> {
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

    fn from_trits(state: TritBuf) -> Self {
        Self {
            state,
            // TODO OPTION
            index: 0,
            _sponge: PhantomData,
        }
    }

    fn to_trits(&self) -> &Trits {
        &self.state
    }
}
