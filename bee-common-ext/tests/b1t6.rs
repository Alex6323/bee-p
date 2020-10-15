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

use bee_common_ext::b1t6::encode;

// TODO factorize tests

#[test]
fn decode() {
    let bytes = vec![1u8];
    let str = encode(&bytes)
        .iter_trytes()
        .map(|trit| char::from(trit))
        .collect::<String>();

    assert_eq!(str, "A9");

    let bytes = vec![127u8];
    let str = encode(&bytes)
        .iter_trytes()
        .map(|trit| char::from(trit))
        .collect::<String>();

    assert_eq!(str, "SE");

    let bytes = vec![128u8];
    let str = encode(&bytes)
        .iter_trytes()
        .map(|trit| char::from(trit))
        .collect::<String>();

    assert_eq!(str, "GV");

    let bytes = vec![255u8];
    let str = encode(&bytes)
        .iter_trytes()
        .map(|trit| char::from(trit))
        .collect::<String>();

    assert_eq!(str, "Z9");

    let bytes = vec![0u8, 1u8];
    let str = encode(&bytes)
        .iter_trytes()
        .map(|trit| char::from(trit))
        .collect::<String>();

    assert_eq!(str, "99A9");

    let bytes = vec![
        0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8,
        0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8,
        0u8, 1u8, 0u8, 1u8, 0u8, 1u8,
    ];
    let str = encode(&bytes)
        .iter_trytes()
        .map(|trit| char::from(trit))
        .collect::<String>();

    assert_eq!(
        str,
        "99A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A999A9"
    );

    let bytes = hex::decode("0001027e7f8081fdfeff").unwrap();
    let str = encode(&bytes)
        .iter_trytes()
        .map(|trit| char::from(trit))
        .collect::<String>();

    assert_eq!(str, "99A9B9RESEGVHVX9Y9Z9");

    let bytes = hex::decode("9ba06c78552776a596dfe360cc2b5bf644c0f9d343a10e2e71debecd30730d03").unwrap();
    let str = encode(&bytes)
        .iter_trytes()
        .map(|trit| char::from(trit))
        .collect::<String>();

    assert_eq!(str, "GWLW9DLDDCLAJDQXBWUZYZODBYPBJCQ9NCQYT9IYMBMWNASBEDTZOYCYUBGDM9C9");
}
