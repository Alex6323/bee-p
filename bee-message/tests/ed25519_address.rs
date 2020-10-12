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

use bee_message::prelude::{Address, Ed25519Address};

#[test]
fn generate_address() {
    let address = Address::from(Ed25519Address::new([1; 32]));
    match address {
        Address::Ed25519(a) => assert_eq!(a.len(), 32),
        _ => panic!("Expect Ed25519 address"),
    }
}

#[test]
fn generate_bech32_string() {
    let mut bytes = [0; 32];
    let vec = hex::decode("52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649").unwrap();
    bytes.copy_from_slice(&vec);
    let address = Ed25519Address::new(bytes);
    let bech32_string = address.to_bech32();
    assert_eq!(
        bech32_string,
        "iot1q9f0mlq8yxpx2nck8a0slxnzr4ef2ek8f5gqxlzd0wasgp73utryjtzcp98"
    );
}

#[test]
fn to_string() {
    let mut bytes = [0; 32];
    let vec = hex::decode("52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649").unwrap();
    bytes.copy_from_slice(&vec);
    let address = Ed25519Address::new(bytes);
    assert_eq!(
        address.to_string(),
        "iot1q9f0mlq8yxpx2nck8a0slxnzr4ef2ek8f5gqxlzd0wasgp73utryjtzcp98"
    );
}
