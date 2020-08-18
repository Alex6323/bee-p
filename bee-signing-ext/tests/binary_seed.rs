use bee_signing_ext::{Signer, Verifier};
use bee_signing_ext::binary::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Seed};

#[test]
fn test_new_seed() {
    let seed = Ed25519Seed::rand();

    assert_ne!(seed.as_bytes(), Ed25519Seed::rand().as_bytes());

    assert_eq!(
        Ed25519PrivateKey::generate_from_seed(&seed, "m").unwrap().as_bytes(),
        Ed25519PrivateKey::generate_from_seed(&seed, "m").unwrap().as_bytes()
    );
    assert_eq!(
        Ed25519PrivateKey::generate_from_seed(&seed, "m/0H/1H/2H/2H/1000000000H").unwrap().as_bytes(),
        Ed25519PrivateKey::generate_from_seed(&seed, "m/0H/1H/2H/2H/1000000000H").unwrap().as_bytes()
    );
}

#[test]
fn invalid_seed_length() {
    let _seed = Ed25519Seed::from_bytes(b"bytes too short").unwrap_err();
}

#[test]
fn invalid_private_key_length() {
    let _key = Ed25519PrivateKey::from_bytes(b"bytes too short").unwrap_err();
}

#[test]
fn invalid_public_key_length() {
    let _key = Ed25519PublicKey::from_bytes(b"bytes too short").unwrap_err();
}

#[test]
fn to_bytes_from_bytes() {
    let seed1 = Ed25519Seed::rand();
    let seed2 = Ed25519Seed::from_bytes(&seed1.to_bytes()).unwrap();

    assert_eq!(seed1.as_bytes(), seed2.as_bytes());

    let private_key1 = Ed25519PrivateKey::generate_from_seed(&seed1, "m/0H/2147483647H/1H/2147483646H/2H").unwrap();
    let private_key2 = Ed25519PrivateKey::from_bytes(&private_key1.to_bytes()).unwrap();

    assert_eq!(private_key1.as_bytes(), private_key2.as_bytes());

    let public_key1 = private_key1.generate_public_key();
    let public_key2 = Ed25519PublicKey::from_bytes(&public_key1.to_bytes()).unwrap();

    assert_eq!(public_key1.as_bytes(), public_key2.as_bytes());
}

#[test]
fn seed_sign_and_verify() {
    let seed = Ed25519Seed::rand();
    let private_key = Ed25519PrivateKey::generate_from_seed(&seed, "m/0H/2147483647H/1H/2147483646H/2H").unwrap();
    let public_key = private_key.generate_public_key();
    let signature = private_key.sign(&[1, 3, 3, 8]);
    public_key.verify(&[1, 3, 3, 8], &signature).unwrap();
}
