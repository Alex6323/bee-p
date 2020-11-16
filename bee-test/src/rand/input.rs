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

use crate::rand::output::random_output_id;

use bee_message::payload::transaction::{Input, UTXOInput};

pub fn random_input() -> Input {
    // TODO add other kind of inputs
    random_utxo_input().into()
}

pub fn random_utxo_input() -> UTXOInput {
    random_output_id().into()
}
