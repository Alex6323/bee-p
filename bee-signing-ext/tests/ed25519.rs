use bee_signing_ext::{Signer, Verifier};

use bee_signing_ext::binary::{BIP32Path, Ed25519PrivateKey, Ed25519Seed};

#[test]
pub fn verify_signature() {
    // Create a rnadome seed
    let seed = Ed25519Seed::rand();
    // ...and a path of wallet account address
    let path = BIP32Path::from_str("m/0'/0'/0'").unwrap();

    // Create the private key from seed
    let private_key = Ed25519PrivateKey::generate_from_seed(&seed, &path).unwrap();
    // ...and create public key from the private key
    let public_key = private_key.generate_public_key();

    // Assume we have a message called "Hello, world!" needs to be singed
    let message = "Hello, world!";
    // We use private key to sign the message.
    let signature = private_key.sign(message.as_bytes());

    // Finally, we use public key to verify the signature.
    public_key.verify(message.as_bytes(), &signature).unwrap();
}
