use bee_signing_ext::binary::{PrivateKey, PublicKey, Seed};

#[test]
fn test_new_seed() {
    let seed = Seed::rand();

    assert_ne!(seed.as_bytes(), Seed::rand().as_bytes());

    assert_eq!(
        PrivateKey::generate_from_seed(&seed, 0).unwrap().as_bytes(),
        PrivateKey::generate_from_seed(&seed, 0).unwrap().as_bytes()
    );
    assert_eq!(
        PrivateKey::generate_from_seed(&seed, 1337).unwrap().as_bytes(),
        PrivateKey::generate_from_seed(&seed, 1337).unwrap().as_bytes()
    );
}

#[test]
fn invalid_seed_length() {
    let _seed = Seed::from_bytes(b"bytes too short").unwrap_err();
}

#[test]
fn invalid_private_key_length() {
    let _key = PrivateKey::from_bytes(b"bytes too short").unwrap_err();
}

#[test]
fn invalid_public_key_length() {
    let _key = PublicKey::from_bytes(b"bytes too short").unwrap_err();
}

#[test]
fn to_bytes_from_bytes() {
    let seed1 = Seed::rand();
    let seed2 = Seed::from_bytes(&seed1.to_bytes()).unwrap();

    assert_eq!(seed1.as_bytes(), seed2.as_bytes());

    let private_key1 = PrivateKey::generate_from_seed(&seed1, 7).unwrap();
    let private_key2 = PrivateKey::from_bytes(&private_key1.to_bytes()).unwrap();

    assert_eq!(private_key1.as_bytes(), private_key2.as_bytes());

    let public_key1 = private_key1.generate_public_key();
    let public_key2 = PublicKey::from_bytes(&public_key1.to_bytes()).unwrap();

    assert_eq!(public_key1.as_bytes(), public_key2.as_bytes());
}

#[test]
fn seed_sign_and_verify() {
    let seed = Seed::rand();
    let private_key = PrivateKey::generate_from_seed(&seed, 7).unwrap();
    let public_key = private_key.generate_public_key();
    let signature = private_key.sign(&[1, 3, 3, 8]).unwrap();
    public_key.verify(&[1, 3, 3, 8], &signature).unwrap();
}
