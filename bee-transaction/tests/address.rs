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

use bee_transaction::prelude::Address;

use hex_literal::hex;

#[test]
fn generate_address() {
    let address = Address::from_ed25519_bytes(&[1; 32]);
    match address {
        Address::Ed25519(a) => assert_eq!(a.len(), 32),
        _ => panic!("Expect Ed25519 address"),
    }
}

#[test]
fn generate_bech32_string() {
    let address = Address::Ed25519(hex!("52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649"));
    let bech32_string = address.to_bech32_string();
    assert_eq!(
        bech32_string,
        "iot1q9f0mlq8yxpx2nck8a0slxnzr4ef2ek8f5gqxlzd0wasgp73utryjtzcp98"
    );
}
