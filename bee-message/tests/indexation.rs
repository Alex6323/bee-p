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

use bee_common::packable::Packable;
use bee_message::prelude::*;

#[test]
fn pack_unpack() {
    let indexation_1 = Indexation::new(
        "indexation".to_string(),
        &[0x42, 0xff, 0x84, 0xa2, 0x42, 0xff, 0x84, 0xa2],
    );
    let bytes = indexation_1.pack_new().unwrap();
    let indexation_2 = Indexation::unpack(&mut bytes.as_slice()).unwrap();

    assert_eq!(indexation_1.packed_len(), bytes.len());
    assert_eq!(indexation_1.index(), indexation_2.index());
    assert_eq!(indexation_1.data(), indexation_2.data());
    assert_eq!(indexation_1.hash(), indexation_2.hash())
}
