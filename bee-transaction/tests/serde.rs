// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use bee_transaction::prelude::*;

use std::num::NonZeroU64;

#[test]
fn input_to_json_serde() {
    let expected = Input::from(UTXOInput::new(Hash::new([1u8; 32]), 2));
    let json = serde_json::to_string(&expected).unwrap();
    let actual = serde_json::from_str(&json).unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn output_to_json_serde() {
    let address = Address::from(Ed25519Address::new([2; 32]));
    let expected = Output::from(SigLockedSingleDeposit::new(
        address,
        NonZeroU64::new(18446744073709551615).unwrap(),
    ));
    let json = serde_json::to_string(&expected).unwrap();
    let actual = serde_json::from_str(&json).unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn unsigned_transaction_to_json_serde() {
    let mut inputs = Vec::new();
    inputs.push(Input::from(UTXOInput::new(Hash::new([1u8; 32]), 2)));
    inputs.push(Input::from(UTXOInput::new(Hash::new([3u8; 32]), 4)));
    let address = Address::from(Ed25519Address::new([2; 32]));
    let mut outputs = Vec::new();
    outputs.push(Output::from(SigLockedSingleDeposit::new(
        address,
        NonZeroU64::new(18446744073709551615).unwrap(),
    )));
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
    let expected = UnlockBlock::from(ReferenceUnlock::from(123));
    let json = serde_json::to_string(&expected).unwrap();
    let actual = serde_json::from_str(&json).unwrap();
    assert_eq!(expected, actual);

    // test wots signature unlock
    let expected = UnlockBlock::from(SignatureUnlock::from(WotsSignature(vec![1; 49])));
    let json = serde_json::to_string(&expected).unwrap();
    let actual = serde_json::from_str(&json).unwrap();
    assert_eq!(expected, actual);

    // test ed25519 signature unlock
    let expected = UnlockBlock::from(SignatureUnlock::from(Ed25519Signature {
        public_key: [2; 32],
        signature: vec![3; 33],
    }));
    let json = serde_json::to_string(&expected).unwrap();
    let actual = serde_json::from_str(&json).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn transaction_message_to_json_serde() {
    // Create single transaction payload first
    let mut inputs = Vec::new();
    inputs.push(Input::from(UTXOInput::new(Hash::new([3u8; 32]), 4)));
    let address = Address::from(Ed25519Address::new([2; 32]));
    let mut outputs = Vec::new();
    outputs.push(Output::from(SigLockedSingleDeposit::new(
        address,
        NonZeroU64::new(18446744073709551615).unwrap(),
    )));
    let unsigned_transaction = UnsignedTransaction {
        inputs,
        outputs,
        payload: None,
    };
    let unlock_blocks = vec![UnlockBlock::from(SignatureUnlock::from(Ed25519Signature {
        public_key: [2; 32],
        signature: vec![3; 33],
    }))];
    let signed = SignedTransaction {
        unsigned_transaction,
        unlock_block_count: 1,
        unlock_blocks,
    };

    // Create a message from signed transaction payload.
    let expected = Message::builder()
        .tips((Hash::new([0; 32]), Hash::new([0; 32])))
        .payload(Payload::SignedTransaction(Box::new(signed)))
        .build()
        .unwrap();

    // test message on serde json
    let json = serde_json::to_string(&expected).unwrap();
    let actual = serde_json::from_str(&json).unwrap();
    assert_eq!(expected, actual);
}
