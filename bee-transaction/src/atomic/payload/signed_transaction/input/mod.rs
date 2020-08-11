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

mod utxo;

pub use utxo::UTXOInput;

use serde::{ser::SerializeStruct, Serialize, Serializer};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Input {
    UTXO(UTXOInput),
}

impl Serialize for Input {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Input::UTXO(UTXOInput {
                ref transaction_id,
                ref output_index,
            }) => {
                let mut serializer = serializer.serialize_struct("Input", 3)?;
                serializer.serialize_field("Input Type", &0u8)?;
                serializer.serialize_field("Transaction ID", transaction_id)?;
                serializer.serialize_field("Transaction Output Index", output_index)?;
                serializer.end()
            }
        }
    }
}
