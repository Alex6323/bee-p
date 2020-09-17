use bee_transaction::prelude::*;

#[test]
fn input_to_json_serde() {
    let expected = Input::new(Hash([1u8; 32]), 2);
    let json = serde_json::to_string(&expected).unwrap();
    let actual = serde_json::from_str(&json).unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn output_to_json_serde() {
    let address = Address::from_ed25519_bytes([2; 32]);
    let expected = Output::new(address, 18446744073709551615);
    let json = serde_json::to_string(&expected).unwrap();
    let actual = serde_json::from_str(&json).unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn unsigned_transaction_to_json_serde() {
    let mut inputs = Vec::new();
    inputs.push(Input::new(Hash([1u8; 32]), 2));
    inputs.push(Input::new(Hash([3u8; 32]), 4));
    let address = Address::from_ed25519_bytes([2; 32]);
    let mut outputs = Vec::new();
    outputs.push(Output::new(address, 18446744073709551615));
    let expected = UnsignedTransaction {
        inputs,
        outputs,
        payload: None,
    };
    let json = serde_json::to_string(&expected).unwrap();
    let actual = serde_json::from_str(&json).unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn unlock_block_to_json_serde() {
    // test reference unlock
    let expected = UnlockBlock::from_reference_unlock(123);
    let json = serde_json::to_string(&expected).unwrap();
    let actual = serde_json::from_str(&json).unwrap();
    assert_eq!(expected, actual);

    // test wots signature unlock
    let expected = UnlockBlock::from_wots_signature(WotsSignature(vec![1; 49]));
    let json = serde_json::to_string(&expected).unwrap();
    let actual = serde_json::from_str(&json).unwrap();
    assert_eq!(expected, actual);

    // test ed25519 signature unlock
    let expected = UnlockBlock::from_ed25519_signature(Ed25519Signature {
        public_key: [2; 32],
        signature: vec![3; 33],
    });
    let json = serde_json::to_string(&expected).unwrap();
    let actual = serde_json::from_str(&json).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn transaction_message_to_json_serde() {
    // Create single transaction payload first
    let mut inputs = Vec::new();
    inputs.push(Input::new(Hash([3u8; 32]), 4));
    let address = Address::from_ed25519_bytes([2; 32]);
    let mut outputs = Vec::new();
    outputs.push(Output::new(address, 18446744073709551615));
    let unsigned_transaction = UnsignedTransaction {
        inputs,
        outputs,
        payload: None,
    };
    let unlock_blocks = vec![UnlockBlock::from_ed25519_signature(Ed25519Signature {
        public_key: [2; 32],
        signature: vec![3; 33],
    })];
    let signed = SignedTransaction {
        unsigned_transaction,
        unlock_block_count: 1,
        unlock_blocks,
    };

    // Create a message from signed transaction payload.
    let expected = Message {
        trunk: Hash([0; 32]),
        branch: Hash([0; 32]),
        payload: Payload::SignedTransaction(Box::new(signed)),
        nonce: 123321,
    };

    // test message on serde json
    let json = serde_json::to_string(&expected).unwrap();
    let actual = serde_json::from_str(&json).unwrap();
    assert_eq!(expected, actual);
}
