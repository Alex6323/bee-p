use super::Seed;
use super::{
    slice_eq, PrivateKey, PrivateKeyGenerator, PublicKey, RecoverableSignature, Signature,
};
use crypto::{Sponge, Trits, TritsBuf, TritsMut};
use std::marker::PhantomData;

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
    state: TritsBuf,
    _sponge: PhantomData<S>,
}

pub struct WotsPublicKey<S> {
    state: TritsBuf,
    _sponge: PhantomData<S>,
}

pub struct WotsSignature<S> {
    state: TritsBuf,
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
    type PrivateKey = WotsPrivateKey<S>;

    fn generate(&self, seed: &Seed, index: u64) -> Self::PrivateKey {
        let subseed = seed.subseed::<S>(index);
        let mut sponge = S::default();
        let mut state = TritsBuf::with_capacity(self.security_level as usize * 6561);

        sponge.absorb(&subseed.as_trits()).unwrap();
        sponge.squeeze_into(&mut state.as_trits_mut());
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
        let mut digests = TritsBuf::with_capacity((self.state.len() / 6561) * 243);
        let mut hash = TritsBuf::with_capacity(243);

        for chunk in hashed_private_key.inner_mut().chunks_mut(243) {
            for _ in 0..26 {
                sponge.absorb(&Trits::from_i8_unchecked(chunk)).unwrap();
                sponge.squeeze_into(&mut TritsMut::from_i8_unchecked(chunk));
                sponge.reset();
            }
        }

        for (i, chunk) in hashed_private_key.inner_ref().chunks(6561).enumerate() {
            sponge.absorb(&Trits::from_i8_unchecked(chunk)).unwrap();
            sponge.squeeze_into(&mut TritsMut::from_i8_unchecked(
                &mut digests.inner_mut()[i * 243..(i + 1) * 243],
            ));
            sponge.reset();
        }

        sponge.absorb(&digests.as_trits()).unwrap();
        sponge.squeeze_into(&mut hash.as_trits_mut());
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

        for (i, chunk) in signature.inner_mut().chunks_mut(243).enumerate() {
            let val = message[i * 3] + message[i * 3 + 1] * 3 + message[i * 3 + 2] * 9;

            for _ in 0..(13 - val) {
                sponge.absorb(&Trits::from_i8_unchecked(chunk)).unwrap();
                sponge.squeeze_into(&mut TritsMut::from_i8_unchecked(chunk));
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
        slice_eq(
            &signature.recover_public_key(message).state.inner_ref(),
            &self.state.inner_ref(),
        )
    }

    fn from_bytes(bytes: &[i8]) -> Self {
        Self {
            state: TritsBuf::from_i8_unchecked(bytes),
            _sponge: PhantomData,
        }
    }

    fn to_bytes(&self) -> &[i8] {
        &self.state.inner_ref()
    }
}

// TODO default impl ?
impl<S: Sponge + Default> Signature for WotsSignature<S> {
    fn size(&self) -> usize {
        self.state.len()
    }

    fn from_bytes(bytes: &[i8]) -> Self {
        Self {
            state: TritsBuf::from_i8_unchecked(bytes),
            _sponge: PhantomData,
        }
    }

    fn to_bytes(&self) -> &[i8] {
        &self.state.inner_ref()
    }
}

impl<S: Sponge + Default> RecoverableSignature for WotsSignature<S> {
    type PublicKey = WotsPublicKey<S>;

    fn recover_public_key(&self, message: &[i8]) -> Self::PublicKey {
        let mut sponge = S::default();
        let mut hash = TritsBuf::with_capacity(243);
        // let mut digests = vec![0; (self.state.len() / 6561) * 243];
        let mut digests = TritsBuf::with_capacity((self.state.len() / 6561) * 243);
        let mut state = self.state.clone();

        for (i, chunk) in state.inner_mut().chunks_mut(243).enumerate() {
            let val = message[i * 3] + message[i * 3 + 1] * 3 + message[i * 3 + 2] * 9;

            for _ in 0..(val - -13) {
                sponge.absorb(&Trits::from_i8_unchecked(chunk)).unwrap();
                sponge.squeeze_into(&mut TritsMut::from_i8_unchecked(chunk));
                sponge.reset();
            }
        }

        for (i, chunk) in state.inner_ref().chunks(6561).enumerate() {
            sponge.absorb(&Trits::from_i8_unchecked(chunk)).unwrap();
            sponge.squeeze_into(&mut TritsMut::from_i8_unchecked(
                &mut digests.inner_mut()[i * 243..(i + 1) * 243],
            ));
            sponge.reset();
        }

        sponge.absorb(&digests.as_trits()).unwrap();
        sponge.squeeze_into(&mut hash.as_trits_mut());
        sponge.reset();

        Self::PublicKey {
            state: hash,
            _sponge: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crypto::{CurlP27, CurlP81};
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
