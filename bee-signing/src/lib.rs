pub mod ed25519;
pub mod mss;
pub mod seed;
pub mod wots;

pub use seed::Seed;

// TODO: documentation
pub trait PrivateKeyGenerator {
    /// The type of the generated private keys
    type PrivateKey;

    /// Deterministically generates and returns a private key
    ///
    /// # Parameters
    ///
    /// * `seed`    A seed to deterministically derive a private key from
    /// * `index`   An index to deterministically derive a private key from
    ///
    /// # Example
    ///
    /// ```
    /// use iota_crypto::Kerl;
    /// use signing::PrivateKeyGenerator;
    /// use signing::seed::Seed;
    /// use signing::wots::WotsPrivateKeyGeneratorBuilder;
    ///
    /// let seed = Seed::new();
    /// let private_key_generator = WotsPrivateKeyGeneratorBuilder::<Kerl>::default().security_level(2).build().unwrap();
    /// let private_key = private_key_generator.generate(&seed, 0);
    /// ```
    fn generate(&self, seed: &Seed, index: u64) -> Self::PrivateKey;
}

// TODO: documentation
pub trait PrivateKey {
    /// The type of the matching public key
    type PublicKey;
    /// The type of the generated signatures
    type Signature;

    /// Returns the public counterpart of a private key
    ///
    /// # Example
    ///
    /// ```
    /// # use iota_crypto::Kerl;
    /// # use signing::PrivateKeyGenerator;
    /// # use signing::seed::Seed;
    /// # use signing::wots::WotsPrivateKeyGeneratorBuilder;
    /// use signing::PrivateKey;
    ///
    /// # let seed = Seed::new();
    /// # let private_key_generator = WotsPrivateKeyGeneratorBuilder::<Kerl>::default().security_level(2).build().unwrap();
    /// # let private_key = private_key_generator.generate(&seed, 0);
    /// let public_key = private_key.generate_public_key();
    /// ```
    fn generate_public_key(&self) -> Self::PublicKey;

    /// Generates and returns a signature for a given message
    ///
    /// # Parameters
    ///
    /// * `message` A slice that holds a message to be signed
    ///
    /// # Example
    ///
    /// ```
    /// # use iota_crypto::Kerl;
    /// # use signing::PrivateKeyGenerator;
    /// # use signing::seed::Seed;
    /// # use signing::wots::WotsPrivateKeyGeneratorBuilder;
    /// use signing::PrivateKey;
    /// use iota_conversion::Trinary;
    ///
    /// # let seed = Seed::new();
    /// # let private_key_generator = WotsPrivateKeyGeneratorBuilder::<Kerl>::default().security_level(2).build().unwrap();
    /// # let mut private_key = private_key_generator.generate(&seed, 0);
    /// let message = "CHXHLHQLOPYP9NSUXTMWWABIBSBLUFXFRNWOZXJPVJPBCIDI99YBSCFYILCHPXHTSEYSYWIGQFERCRVDD".trits();
    /// let signature = private_key.sign(&message);
    /// ```
    fn sign(&mut self, message: &[i8]) -> Self::Signature;
}

// TODO: documentation
pub trait PublicKey {
    // TODO: documentation
    type Signature;

    // TODO: documentation
    fn verify(&self, message: &[i8], signature: &Self::Signature) -> bool;

    // TODO: documentation
    fn from_bytes(bytes: &[i8]) -> Self;

    // TODO: documentation
    fn to_bytes(&self) -> &[i8];
}

// TODO: documentation
pub trait Signature {
    // TODO: documentation
    fn size(&self) -> usize;

    // TODO: documentation
    fn from_bytes(bytes: &[i8]) -> Self;

    // TODO: documentation
    fn to_bytes(&self) -> &[i8];
}

// TODO: documentation
pub trait RecoverableSignature {
    // TODO: documentation
    type PublicKey;

    // TODO: documentation
    fn recover_public_key(&self, message: &[i8]) -> Self::PublicKey;
}

// TODO: remove
// TODO: documentation
pub fn slice_eq(xs: &[i8], ys: &[i8]) -> bool {
    for (x, y) in xs.iter().zip(ys.iter()) {
        if x != y {
            return false;
        }
    }

    true
}
