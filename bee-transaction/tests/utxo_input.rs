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

use bee_transaction::prelude::{MessageId, UTXOInput};

#[test]
fn to_string() {
    let mut bytes = [0; 32];
    let vec = hex::decode("52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649").unwrap();
    bytes.copy_from_slice(&vec);
    let message_id = MessageId::new(bytes);
    let utxo_input = UTXOInput::new(message_id, 1).unwrap();
    assert_eq!(
        utxo_input.to_string(),
        "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6490100"
    );
}
