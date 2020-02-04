use crate::{
    IotaSeed, PrivateKey, PrivateKeyGenerator, PublicKey, RecoverableSignature, Seed,
    Signature,
};

use bee_crypto::Sponge;
use bee_ternary::{Trits, TritsBuf, TritsMut};

use std::marker::PhantomData;
use ternary::{Trits, TritBuf};

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
    state: TritBuf,
    _sponge: PhantomData<S>,
}

pub struct WotsPublicKey<S> {
    state: TritBuf,
    _sponge: PhantomData<S>,
}

pub struct WotsSignature<S> {
    state: TritBuf,
    _sponge: PhantomData<S>,
}

// TODO: documentation
#[derive(Debug, PartialEq)]
pub enum WotsError {
    InvalidSecurityLevel(u8),
    MissingSecurityLevel,
    FailedSpongeOperation,
}

impl<S: Sponge + Default> WotsPrivateKeyGeneratorBuilder<S> {
    pub fn security_level(&mut self, security_level: u8) -> &mut Self {
        self.security_level = Some(security_level);
        self
    }

    pub fn build(&mut self) -> Result<WotsPrivateKeyGenerator<S>, WotsError> {
        let security_level = match self.security_level {
            Some(security_level) => match security_level {
                1 | 2 | 3 => security_level,
                _ => return Err(WotsError::InvalidSecurityLevel(security_level)),
            },
            None => return Err(WotsError::MissingSecurityLevel),
        };

        Ok(WotsPrivateKeyGenerator {
            security_level: security_level,
            _sponge: PhantomData,
        })
    }
}

impl<S: Sponge + Default> PrivateKeyGenerator for WotsPrivateKeyGenerator<S> {
    type Seed = IotaSeed<S>;
    type PrivateKey = WotsPrivateKey<S>;
    type Error = WotsError;

    fn generate(&self, seed: &Self::Seed, index: u64) -> Result<Self::PrivateKey, Self::Error> {
        let subseed = seed.subseed(index);
        let mut sponge = S::default();
        let mut state = TritBuf::zeros(self.security_level as usize * 6561);

        if let Err(_) = sponge.digest_into(
            subseed.trits(),
            &mut state,
        ) {
            return Err(Self::Error::FailedSpongeOperation);
        }

        Ok(Self::PrivateKey {
            state: state,
            _sponge: PhantomData,
        })
    }
}

impl<S: Sponge + Default> PrivateKey for WotsPrivateKey<S> {
    type PublicKey = WotsPublicKey<S>;
    type Signature = WotsSignature<S>;
    type Error = WotsError;

    fn generate_public_key(&self) -> Result<Self::PublicKey, Self::Error> {
        let mut sponge = S::default();
        let mut hashed_private_key = self.state.clone();
        let mut digests: TritBuf = TritBuf::zeros((self.state.len() / 6561) * 243);
        let mut hash = TritBuf::zeros(243);

        for chunk in hashed_private_key.chunks_mut(243) {
            for _ in 0..26 {
                if let Err(_) = sponge.absorb(chunk) {
                    return Err(Self::Error::FailedSpongeOperation);
                }
                sponge.squeeze_into(chunk);
                sponge.reset();
            }
        }

        for (i, chunk) in hashed_private_key.chunks(6561).enumerate() {
            if let Err(_) = sponge.digest_into(
                chunk,
                &mut digests[i * 243..(i + 1) * 243],
            ) {
                return Err(Self::Error::FailedSpongeOperation);
            }
        }

        if let Err(_) = sponge.digest_into(&digests, &mut hash) {
            return Err(Self::Error::FailedSpongeOperation);
        }

        Ok(Self::PublicKey {
            state: hash,
            _sponge: PhantomData,
        })
    }

    // TODO: enforce hash size ?
    fn sign(&mut self, message: &[i8]) -> Result<Self::Signature, Self::Error> {
        let mut sponge = S::default();
        let mut signature = self.state.clone();

        for (i, chunk) in signature.chunks_mut(243).enumerate() {
            let val = message[i * 3] + message[i * 3 + 1] * 3 + message[i * 3 + 2] * 9;

            for _ in 0..(13 - val) {
                if let Err(_) = sponge.absorb(chunk) {
                    return Err(Self::Error::FailedSpongeOperation);
                }
                sponge.squeeze_into(chunk);
                sponge.reset();
            }
        }

        Ok(Self::Signature {
            state: signature,
            _sponge: PhantomData,
        })
    }
}

impl<S: Sponge + Default> PublicKey for WotsPublicKey<S> {
    type Signature = WotsSignature<S>;
    type Error = WotsError;

    // TODO: enforce hash size ?
    fn verify(&self, message: &[i8], signature: &Self::Signature) -> Result<bool, Self::Error> {
        Ok(&signature.recover_public_key(message)?.state == &self.state)
    }

    fn from_buf(state: TritBuf) -> Self {
        Self {
            state,
            _sponge: PhantomData,
        }
    }

    fn as_bytes(&self) -> &[i8] {
        self.state.as_i8_slice()
    }

    fn trits(&self) -> &Trits {
        &self.state
    }
}

// TODO default impl ?
impl<S: Sponge + Default> Signature for WotsSignature<S> {
    fn size(&self) -> usize {
        self.state.len()
    }

    fn from_buf(state: TritBuf) -> Self {
        Self {
            state,
            _sponge: PhantomData,
        }
    }

    fn as_bytes(&self) -> &[i8] {
        self.state.as_i8_slice()
    }

    fn trits(&self) -> &Trits {
        &self.state
    }
}

impl<S: Sponge + Default> RecoverableSignature for WotsSignature<S> {
    type PublicKey = WotsPublicKey<S>;
    type Error = WotsError;

    fn recover_public_key(&self, message: &[i8]) -> Result<Self::PublicKey, Self::Error> {
        let mut sponge = S::default();
        let mut hash = TritBuf::zeros(243);
        // let mut digests = vec![0; (self.state.len() / 6561) * 243];
        let mut digests: TritBuf = TritBuf::zeros((self.state.len() / 6561) * 243);
        let mut state = self.state.clone();

        for (i, chunk) in state.chunks_mut(243).enumerate() {
            let val = message[i * 3] + message[i * 3 + 1] * 3 + message[i * 3 + 2] * 9;

            for _ in 0..(val - -13) {
                if let Err(_) = sponge.absorb(chunk) {
                    return Err(Self::Error::FailedSpongeOperation);
                }
                sponge.squeeze_into(chunk);
                sponge.reset();
            }
        }

        for (i, chunk) in state.chunks(6561).enumerate() {
            if let Err(_) = sponge.digest_into(
                chunk,
                &mut digests[i * 243..(i + 1) * 243],
            ) {
                return Err(Self::Error::FailedSpongeOperation);
            }
        }

        if let Err(_) = sponge.digest_into(&digests, &mut hash) {
            return Err(Self::Error::FailedSpongeOperation);
        }

        Ok(Self::PublicKey {
            state: hash,
            _sponge: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use bee_crypto::{CurlP27, CurlP81};
    use iota_conversion::Trinary;

    const SEED: &str =
        "NNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNN";
    const MESSAGE: &str =
        "CHXHLHQLOPYP9NSUXTMWWABIBSBLUFXFRNWOZXJPVJPBCIDI99YBSCFYILCHPXHTSEYSYWIGQFERCRVDD";

    // #[test]
    // fn wots_generator_missing_security_level_test() {
    //     match WotsPrivateKeyGeneratorBuilder::<Kerl>::default().build() {
    //         Ok(_) => unreachable!(),
    //         Err(err) => assert_eq!(err, WotsError::MissingSecurityLevel),
    //     }
    // }

    // #[test]
    // fn wots_generator_invalid_security_level_test() {
    //     match WotsPrivateKeyGeneratorBuilder::<Kerl>::default()
    //         .security_level(0)
    //         .build()
    //     {
    //         Err(WotsError::InvalidSecurityLevel(s)) => assert_eq!(s, 0),
    //         _ => unreachable!(),
    //     }
    //
    //     match WotsPrivateKeyGeneratorBuilder::<Kerl>::default()
    //         .security_level(4)
    //         .build()
    //     {
    //         Err(WotsError::InvalidSecurityLevel(s)) => assert_eq!(s, 4),
    //         _ => unreachable!(),
    //     }
    // }

    // #[test]
    // fn wots_generator_valid_test() {
    //     for s in 1..4 {
    //         assert_eq!(
    //             WotsPrivateKeyGeneratorBuilder::<Kerl>::default()
    //                 .security_level(s)
    //                 .build()
    //                 .is_ok(),
    //             true
    //         );
    //     }
    // }

    fn wots_generic_complete_test<S: Sponge + Default>() {
        let seed = IotaSeed::<S>::from_bytes(&SEED.trits()).unwrap();

        for security in 1..4 {
            for index in 0..5 {
                let private_key_generator = WotsPrivateKeyGeneratorBuilder::<S>::default()
                    .security_level(security)
                    .build()
                    .unwrap();
                let mut private_key = private_key_generator.generate(&seed, index).unwrap();
                let public_key = private_key.generate_public_key().unwrap();
                let signature = private_key.sign(&MESSAGE.trits()).unwrap();
                let recovered_public_key = signature.recover_public_key(&MESSAGE.trits()).unwrap();
                assert_eq!(public_key.to_bytes(), recovered_public_key.to_bytes());
                let valid = public_key.verify(&MESSAGE.trits(), &signature).unwrap();
                assert!(valid);
            }
        }
    }

    // #[test]
    // fn wots_kerl_complete_test() {
    //     wots_generic_complete_test::<Kerl>();
    // }

    #[test]
    fn wots_curl27_complete_test() {
        wots_generic_complete_test::<CurlP27>();
    }

    #[test]
    fn wots_curl81_complete_test() {
        wots_generic_complete_test::<CurlP81>();
    }
}
