mod ed25519;
mod iota_seed;
mod mss;
mod scheme;
mod wots;

pub use iota_seed::IotaSeed;
pub use scheme::{
    PrivateKey, PrivateKeyGenerator, PublicKey, RecoverableSignature, Seed, Signature,
};
pub use wots::{
    WotsPrivateKey, WotsPrivateKeyGenerator, WotsPrivateKeyGeneratorBuilder, WotsPublicKey,
    WotsSignature,
};

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
