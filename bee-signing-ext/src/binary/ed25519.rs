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

//! Binary seed to derive private keys, public keys and signatures from.

use bee_common_derive::{SecretDebug, SecretDisplay, SecretDrop};

use blake2::VarBlake2b;
use digest::{VariableOutput, Update};
use ed25519_dalek::{ExpandedSecretKey, Verifier};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zeroize::Zeroize;

/// Errors occuring during Ed25519 operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Private Key Error
    #[error("Failed to generate private key.")]
    PrivateKeyError,
    /// Verify Error
    #[error("Failed to verify signature.")]
    VerifyError,
}

/// Binary `Ed25519`-based `Seed` to derive private keys, public keys and signatures from.
#[derive(SecretDebug, SecretDisplay, SecretDrop)]
pub struct Seed(ed25519_dalek::SecretKey);

impl Zeroize for Seed {
    fn zeroize(&mut self) {
        self.0.zeroize()
    }
}

impl Seed {
    /// Creates a new random `Seed`.
    pub fn rand() -> Self {
        // `ThreadRng` implements `CryptoRng` so it is safe to use in cryptographic contexts.
        // https://rust-random.github.io/rand/rand/trait.CryptoRng.html
        let mut rng = rand::thread_rng();
        Self(ed25519_dalek::SecretKey::generate(&mut rng))
    }
}

/// Ed25519 private key.
#[derive(SecretDebug, SecretDisplay, SecretDrop)]
pub struct PrivateKey(ed25519_dalek::SecretKey);

impl Zeroize for PrivateKey {
    fn zeroize(&mut self) {
        self.0.zeroize()
    }
}

impl PrivateKey {
    /// Deterministically generates and returns a private key from a seed and an index.
    ///
    /// # Arguments
    ///
    /// * `seed`    A seed to deterministically derive a private key from.
    pub fn generate_from_seed(seed: &Seed, index: u64) -> Result<Self, Error> {
        let mut subseed = seed.0.to_bytes();
        let mut hash = VarBlake2b::new(256).map_err(|_| Error::PrivateKeyError)?;
        hash.update(index.to_le_bytes());
        hash.finalize_variable(|ha| {
            subseed.iter_mut().zip(ha.iter()).for_each(|(i, h)| *i = *i ^ *h);
        });
        Ok(Self(
            ed25519_dalek::SecretKey::from_bytes(&subseed).map_err(|_| Error::PrivateKeyError)?,
        ))
    }

    /// Returns the public counterpart of a private key.
    pub fn generate_public_key(&self) -> PublicKey {
        PublicKey((&self.0).into())
    }

    /// Generates and returns a signature for a given message.
    ///
    /// # Arguments
    ///
    /// * `message` A slice that holds a message to be signed.
    pub fn sign(&mut self, message: &[u8]) -> Result<Signature, Error> {
        let key: ExpandedSecretKey = (&self.0).into();
        Ok(Signature(key.sign(message, &(&self.0).into())))
    }
}

/// Ed25519 public key.
#[derive(Debug, Serialize, Deserialize)]
pub struct PublicKey(ed25519_dalek::PublicKey);

impl PublicKey {
    /// Verifies a signature for a given message.
    ///
    /// # Arguments
    ///
    /// * `message`     A slice that holds a message to verify a signature for.
    /// * `signature`   The signature to verify.
    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<bool, Error> {
        self.0.verify(message, &signature.0).map_err(|_| Error::VerifyError)?;
        Ok(true)
    }
}

/// Ed25519 signature
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Signature(ed25519_dalek::Signature);
