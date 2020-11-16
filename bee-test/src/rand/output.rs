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

use crate::rand::{integer::random_integer_range, transaction::random_transaction_id};

use bee_message::payload::transaction::{OutputId, INPUT_OUTPUT_INDEX_RANGE};

pub fn random_output_id() -> OutputId {
    OutputId::new(random_transaction_id(), random_integer_range(INPUT_OUTPUT_INDEX_RANGE)).unwrap()
}
