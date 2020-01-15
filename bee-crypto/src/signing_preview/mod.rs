// pub mod ed25519;
pub mod mss;
pub mod seed;
pub mod wots;

pub use seed::Seed;

// TODO: documentation
pub trait PrivateKeyGenerator {
    type PrivateKey;

    fn generate(&self, seed: &Seed, index: u64) -> Self::PrivateKey;
}

// TODO: documentation
pub trait PrivateKey {
    type PublicKey;
    type Signature;

    fn generate_public_key(&self) -> Self::PublicKey;

    fn sign(&mut self, message: &[i8]) -> Self::Signature;
}

// TODO: documentation
pub trait PublicKey {
    type Signature;

    fn verify(&self, message: &[i8], signature: &Self::Signature) -> bool;

    fn from_bytes(bytes: &[i8]) -> Self;

    fn to_bytes(&self) -> &[i8];
}

// TODO: documentation
pub trait Signature {
    fn size(&self) -> usize;

    fn from_bytes(bytes: &[i8]) -> Self;

    fn to_bytes(&self) -> &[i8];
}

// TODO: documentation
pub trait RecoverableSignature {
    type PublicKey;

    fn recover_public_key(&self, message: &[i8]) -> Self::PublicKey;
}

// TODO: remove
pub fn slice_eq(xs: &[i8], ys: &[i8]) -> bool {
    for (x, y) in xs.iter().zip(ys.iter()) {
        if x != y {
            return false;
        }
    }

    true
}
