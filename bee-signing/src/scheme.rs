pub trait Seed {
    type Error;

    fn new() -> Self;
    fn from_bytes(bytes: &[i8]) -> Result<Self, Self::Error>
    where
        Self: Sized;
    fn to_bytes(&self) -> &[i8];
}

// TODO: documentation
pub trait PrivateKeyGenerator {
    type Seed: Seed;
    /// The type of the generated private keys
    type PrivateKey: PrivateKey;
    type Error;

    /// Deterministically generates and returns a private key
    ///
    /// # Arguments
    ///
    /// * `seed`    A seed to deterministically derive a private key from
    /// * `index`   An index to deterministically derive a private key from
    ///
    /// # Example
    ///
    /// ```
    /// use crypto::Kerl;
    /// use signing::PrivateKeyGenerator;
    /// use signing::iota_seed::IotaSeed;
    /// use signing::wots::WotsPrivateKeyGeneratorBuilder;
    ///
    /// let seed = IotaSeed::<Kerl>::new();
    /// let private_key_generator = WotsPrivateKeyGeneratorBuilder::<Kerl>::default().security_level(2).build().unwrap();
    /// let private_key = private_key_generator.generate(&seed, 0);
    /// ```
    fn generate(&self, seed: &Self::Seed, index: u64) -> Result<Self::PrivateKey, Self::Error>;
}

// TODO: documentation
pub trait PrivateKey {
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
    /// # use crypto::Kerl;
    /// # use signing::PrivateKeyGenerator;
    /// # use signing::iota_seed::IotaSeed;
    /// # use signing::wots::WotsPrivateKeyGeneratorBuilder;
    /// use signing::PrivateKey;
    ///
    /// # let seed = IotaSeed::<Kerl>::new();
    /// # let private_key_generator = WotsPrivateKeyGeneratorBuilder::<Kerl>::default().security_level(2).build().unwrap();
    /// # let private_key = private_key_generator.generate(&seed, 0);
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
    /// # use crypto::Kerl;
    /// # use signing::PrivateKeyGenerator;
    /// # use signing::iota_seed::IotaSeed;
    /// # use signing::wots::WotsPrivateKeyGeneratorBuilder;
    /// use signing::PrivateKey;
    /// use iota_conversion::Trinary;
    ///
    /// # let seed = IotaSeed::<Kerl>::new();
    /// # let private_key_generator = WotsPrivateKeyGeneratorBuilder::<Kerl>::default().security_level(2).build().unwrap();
    /// # let mut private_key = private_key_generator.generate(&seed, 0);
    /// let message = "CHXHLHQLOPYP9NSUXTMWWABIBSBLUFXFRNWOZXJPVJPBCIDI99YBSCFYILCHPXHTSEYSYWIGQFERCRVDD".trits();
    /// let signature = private_key.sign(&message);
    /// ```
    fn sign(&mut self, message: &[i8]) -> Result<Self::Signature, Self::Error>;
}

// TODO: documentation
pub trait PublicKey {
    // TODO: documentation
    type Signature: Signature;
    type Error;

    // TODO: documentation
    fn verify(&self, message: &[i8], signature: &Self::Signature) -> Result<bool, Self::Error>;

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
pub trait RecoverableSignature: Signature {
    // TODO: documentation
    type PublicKey: PublicKey;
    type Error;

    // TODO: documentation
    fn recover_public_key(&self, message: &[i8]) -> Result<Self::PublicKey, Self::Error>;
}
