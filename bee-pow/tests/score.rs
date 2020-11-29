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

use bee_pow::compute_pow_score;

#[test]
fn score() {
    let message: [u8; 21] = [
        0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x2c, 0x20, 0x57, 0x6f, 0x72, 0x6c, 0x64, 0x21, 0x5e, 0xe6, 0xaa, 0xaa, 0xaa,
        0xaa, 0xaa, 0xaa,
    ];

    assert_eq!(compute_pow_score(&message), 937.2857142857143);

    let message: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];

    assert_eq!(compute_pow_score(&message), 3u128.pow(1) as f64 / 8 as f64);

    let message: [u8; 8] = [203, 124, 2, 0, 0, 0, 0, 0];

    assert_eq!(compute_pow_score(&message), 3u128.pow(10) as f64 / 8 as f64);

    let message: [u8; 8] = [65, 235, 119, 85, 85, 85, 85, 85];

    assert_eq!(compute_pow_score(&message), 3u128.pow(14) as f64 / 8 as f64);

    let message: [u8; 10000] = [0; 10000];

    assert_eq!(compute_pow_score(&message), 3u128.pow(0) as f64 / 10000 as f64);
}
