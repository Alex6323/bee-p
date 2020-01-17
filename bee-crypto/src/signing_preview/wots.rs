// TODO clean use
use super::*;
use iota_crypto::Sponge;
use std::marker::PhantomData;

// TODO state as Vec<i8> ?
// TODO constants

#[derive(Default)]
pub struct WotsPrivateKeyGeneratorBuilder<S> {
    security_level: Option<u8>,
    _sponge: PhantomData<S>,
}

#[derive(Default, Clone, Copy)]
pub struct WotsPrivateKeyGenerator<S> {
    security_level: u8,
    _sponge: PhantomData<S>,
}

pub struct WotsPrivateKey<S> {
    state: Vec<i8>,
    _sponge: PhantomData<S>,
}

pub struct WotsPublicKey<S> {
    state: Vec<i8>,
    _sponge: PhantomData<S>,
}

pub struct WotsSignature<S> {
    state: Vec<i8>,
    _sponge: PhantomData<S>,
}

// TODO: documentation
#[derive(Debug, PartialEq)]
pub enum WotsError {
    InvalidSecurityLevel(u8),
    MissingSecurityLevel,
}

impl<S: Sponge + Default> WotsPrivateKeyGeneratorBuilder<S> {
    pub fn security_level(&mut self, security_level: u8) -> &mut Self {
        self.security_level = Some(security_level);
        self
    }

    pub fn build(&mut self) -> Result<WotsPrivateKeyGenerator<S>, WotsError> {
        match self.security_level {
            Some(security_level) => match security_level {
                1 | 2 | 3 => Ok(WotsPrivateKeyGenerator {
                    security_level: security_level,
                    _sponge: PhantomData,
                }),
                _ => Err(WotsError::InvalidSecurityLevel(security_level)),
            },
            None => Err(WotsError::MissingSecurityLevel),
        }
    }
}

impl<S: Sponge + Default> PrivateKeyGenerator for WotsPrivateKeyGenerator<S> {
    type PrivateKey = WotsPrivateKey<S>;

    fn generate(&self, seed: &Seed, index: u64) -> Self::PrivateKey {
        let subseed = seed.subseed::<S>(index);
        let mut sponge = S::default();
        let mut state = vec![0; self.security_level as usize * 6561];

        sponge.absorb(&subseed.to_bytes()).unwrap();
        sponge
            .squeeze(&mut state[0..self.security_level as usize * 6561])
            .unwrap();
        sponge.reset();

        Self::PrivateKey {
            state: state,
            _sponge: PhantomData,
        }
    }
}

impl<S: Sponge + Default> PrivateKey for WotsPrivateKey<S> {
    type PublicKey = WotsPublicKey<S>;
    type Signature = WotsSignature<S>;

    fn generate_public_key(&self) -> Self::PublicKey {
        let mut sponge = S::default();
        let mut hashed_private_key = self.state.clone();
        let mut digests = vec![0; (self.state.len() / 6561) * 243];
        let mut hash = vec![0; 243];

        for chunk in hashed_private_key.chunks_mut(243) {
            for _ in 0..26 {
                sponge.absorb(chunk).unwrap();
                sponge.squeeze(chunk).unwrap();
                sponge.reset();
            }
        }

        for (i, chunk) in hashed_private_key.chunks(6561).enumerate() {
            sponge.absorb(chunk).unwrap();
            sponge
                .squeeze(&mut digests[i * 243..(i + 1) * 243])
                .unwrap();
            sponge.reset();
        }

        sponge.absorb(&digests).unwrap();
        sponge.squeeze(&mut hash).unwrap();
        sponge.reset();

        Self::PublicKey {
            state: hash,
            _sponge: PhantomData,
        }
    }

    // TODO: enforce hash size ?
    fn sign(&mut self, message: &[i8]) -> Self::Signature {
        let mut sponge = S::default();
        let mut signature = self.state.clone();

        for (i, chunk) in signature.chunks_mut(243).enumerate() {
            let val = message[i * 3] + message[i * 3 + 1] * 3 + message[i * 3 + 2] * 9;

            for _ in 0..(13 - val) {
                sponge.absorb(chunk).unwrap();
                sponge.squeeze(chunk).unwrap();
                sponge.reset();
            }
        }

        Self::Signature {
            state: signature,
            _sponge: PhantomData,
        }
    }
}

/////////////////////////

impl<S: Sponge + Default> PublicKey for WotsPublicKey<S> {
    type Signature = WotsSignature<S>;

    // TODO: enforce hash size ?
    fn verify(&self, message: &[i8], signature: &Self::Signature) -> bool {
        slice_eq(&signature.recover_public_key(message).state, &self.state)
    }

    fn from_bytes(bytes: &[i8]) -> Self {
        Self {
            state: bytes.to_vec(),
            _sponge: PhantomData,
        }
    }

    fn to_bytes(&self) -> &[i8] {
        &self.state
    }
}

// TODO default impl ?
impl<S: Sponge + Default> Signature for WotsSignature<S> {
    fn size(&self) -> usize {
        self.state.len()
    }
    fn from_bytes(bytes: &[i8]) -> Self {
        Self {
            state: bytes.to_vec(),
            _sponge: PhantomData,
        }
    }
    fn to_bytes(&self) -> &[i8] {
        &self.state
    }
}

impl<S: Sponge + Default> RecoverableSignature for WotsSignature<S> {
    type PublicKey = WotsPublicKey<S>;

    fn recover_public_key(&self, message: &[i8]) -> Self::PublicKey {
        let mut sponge = S::default();
        let mut hash = [0; 243];
        let mut state = self.state.clone();
        let mut digests = vec![0; (self.state.len() / 6561) * 243];

        for (i, chunk) in state.chunks_mut(243).enumerate() {
            let val = message[i * 3] + message[i * 3 + 1] * 3 + message[i * 3 + 2] * 9;

            for _ in 0..(val - -13) {
                sponge.absorb(chunk).unwrap();
                sponge.squeeze(chunk).unwrap();
                sponge.reset();
            }
        }

        for (i, chunk) in state.chunks_mut(6561).enumerate() {
            sponge.absorb(&chunk).unwrap();
            sponge
                .squeeze(&mut digests[i * 243..(i + 1) * 243])
                .unwrap();
            sponge.reset();
        }

        sponge.absorb(&digests).unwrap();
        sponge.squeeze(&mut hash).unwrap();
        sponge.reset();

        Self::PublicKey {
            state: hash.to_vec(),
            _sponge: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use iota_conversion::Trinary;
    use iota_crypto::{Curl, Kerl};

    const SEED: &str =
        "NNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNN";
    const MESSAGE: &str =
        "CHXHLHQLOPYP9NSUXTMWWABIBSBLUFXFRNWOZXJPVJPBCIDI99YBSCFYILCHPXHTSEYSYWIGQFERCRVDD";

    #[test]
    fn wots_generator_missing_security_level_test() {
        match WotsPrivateKeyGeneratorBuilder::<Kerl>::default().build() {
            Ok(_) => unreachable!(),
            Err(err) => assert_eq!(err, WotsError::MissingSecurityLevel),
        }
    }

    #[test]
    fn wots_generator_invalid_security_level_test() {
        match WotsPrivateKeyGeneratorBuilder::<Kerl>::default()
            .security_level(0)
            .build()
        {
            Err(WotsError::InvalidSecurityLevel(s)) => assert_eq!(s, 0),
            _ => unreachable!(),
        }

        match WotsPrivateKeyGeneratorBuilder::<Kerl>::default()
            .security_level(4)
            .build()
        {
            Err(WotsError::InvalidSecurityLevel(s)) => assert_eq!(s, 4),
            _ => unreachable!(),
        }
    }

    #[test]
    fn wots_generator_valid_test() {
        for s in 1..4 {
            assert_eq!(
                WotsPrivateKeyGeneratorBuilder::<Kerl>::default()
                    .security_level(s)
                    .build()
                    .is_ok(),
                true
            );
        }
    }

    fn wots_generic_complete_test<S: Sponge + Default>() {
        let seed = Seed::from_bytes(&SEED.trits()).unwrap();

        for security in 1..4 {
            for index in 0..5 {
                let private_key_generator = WotsPrivateKeyGeneratorBuilder::<S>::default()
                    .security_level(security)
                    .build()
                    .unwrap();
                // TODO mut ?
                let mut private_key = private_key_generator.generate(&seed, index);
                let public_key = private_key.generate_public_key();
                let bytes = public_key.to_bytes();
                let signature = private_key.sign(&MESSAGE.trits());
                let recovered_public_key = signature.recover_public_key(&MESSAGE.trits());
                assert!(slice_eq(
                    public_key.to_bytes(),
                    recovered_public_key.to_bytes()
                ));
                let valid = public_key.verify(&MESSAGE.trits(), &signature);
                assert!(valid);
            }
        }
    }

    #[test]
    fn wots_kerl_complete_test() {
        wots_generic_complete_test::<Kerl>();
    }
    #[test]
    fn wots_curl_complete_test() {
        wots_generic_complete_test::<Curl>();
    }
}
