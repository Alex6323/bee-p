use super::seed::Seed;
use super::{PrivateKey, PrivateKeyGenerator, PublicKey, Signature};
use iota_crypto::Kerl;
use rand::rngs::OsRng;

#[derive(Default)]
pub struct Ed25519PrivateKeyGeneratorBuilder {}

pub struct Ed25519PrivateKeyGenerator {}

pub struct Ed25519PrivateKey {
    private_key: ed25519_dalek::SecretKey,
}

pub struct Ed25519PublicKey {
    public_key: ed25519_dalek::PublicKey,
}

pub struct Ed25519Signature {
    public_key: ed25519_dalek::PublicKey,
    signature: ed25519_dalek::Signature,
}

impl Ed25519PrivateKeyGeneratorBuilder {
    pub fn build(&mut self) -> Ed25519PrivateKeyGenerator {
        Ed25519PrivateKeyGenerator {}
    }
}

impl PrivateKeyGenerator for Ed25519PrivateKeyGenerator {
    type PrivateKey = Ed25519PrivateKey;

    fn generate(&self, seed: &Seed, index: u64) -> Self::PrivateKey {
        let mut csprng = OsRng {};
        let private_key = ed25519_dalek::SecretKey::generate(&mut csprng);

        let _subseed = seed.subseed::<Kerl>(index);

        Self::PrivateKey {
            private_key: private_key,
        }
    }
}

impl PrivateKey for Ed25519PrivateKey {
    type PublicKey = Ed25519PublicKey;
    type Signature = Ed25519Signature;

    fn generate_public_key(&self) -> Self::PublicKey {
        Self::PublicKey {
            public_key: (&self.private_key).into(),
        }
    }

    // TODO: hash ? enforce size ?
    fn sign(&mut self, message: &[i8]) -> Self::Signature {
        let test = unsafe { &*(message as *const _ as *const [u8]) };
        let private_key = &self.private_key;
        let public_key = self.generate_public_key();
        let expanded_private_key = ed25519_dalek::ExpandedSecretKey::from(private_key);
        let signature = expanded_private_key.sign(test, &public_key.public_key);

        Self::Signature {
            public_key: public_key.public_key,
            signature: signature,
        }
    }
}

impl PublicKey for Ed25519PublicKey {
    type Signature = Ed25519Signature;

    fn verify(&self, message: &[i8], signature: &Self::Signature) -> bool {
        let test = unsafe { &*(message as *const _ as *const [u8]) };

        self.public_key.verify(test, &signature.signature).is_ok()
    }

    fn from_bytes(bytes: &[i8]) -> Self {
        Self {
            public_key: ed25519_dalek::PublicKey::default(),
        }
    }

    fn to_bytes(&self) -> &[i8] {
        // &self.state
        &[]
    }
}

// impl Ed25519Signature {
//     pub fn new(state: &[i8]) -> Ed25519Signature {
//         Ed25519Signature{
//             state: state.to_vec(),
//         }
//     }
// }

// TODO default impl ?
impl Signature for Ed25519Signature {
    fn size(&self) -> usize {
        // self.state.len()
        42
    }

    fn from_bytes(bytes: &[i8]) -> Self {
        let test = unsafe { &*(bytes as *const _ as *const [u8]) };

        Self {
            public_key: ed25519_dalek::PublicKey::from_bytes(test).unwrap(),
            signature: ed25519_dalek::Signature::from_bytes(test).unwrap(),
        }
    }

    fn to_bytes(&self) -> &[i8] {
        &[]
    }
}

// impl RecoverableSignature for Ed25519Signature {
//     type PublicKey = Ed25519PublicKey;
//
//     fn recover_public_key(&self, message: &[i8]) -> Self::PublicKey {
//
//         Ed25519PublicKey{
//             state: hash.to_vec(),
//         }
//     }
// }

#[cfg(test)]
mod tests {

    use super::*;
    use iota_conversion::Trinary;

    const SEED1: &str =
        "NNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNN";
    const SEED2: &str =
        "NNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNDNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNN";
    const MESSAGE: &str =
        "CHXHLHQLOPYP9NSUXTMWWABIBSBLUFXFRNWOZXJPVJPBCIDI99YBSCFYILCHPXHTSEYSYWIGQFERCRVDD";

    #[test]
    fn ed25519_test() {
        let seed_trits_1 = &SEED1.trits();
        let seed_trits_2 = &SEED2.trits();
        let seed = Seed::from_bytes(&SEED1.trits()).unwrap();

        for index in 0..25 {
            let private_key_generator = Ed25519PrivateKeyGeneratorBuilder::default().build();
            // TODO mut ?
            let mut private_key = private_key_generator.generate(&seed, index);
            let public_key = private_key.generate_public_key();
            let signature_good = private_key.sign(seed_trits_1);
            let signature_bad = private_key.sign(seed_trits_2);
            let mut valid = public_key.verify(seed_trits_1, &signature_good);
            assert!(valid);
            valid = public_key.verify(seed_trits_2, &signature_good);
            assert!(!valid);
        }
    }
}
