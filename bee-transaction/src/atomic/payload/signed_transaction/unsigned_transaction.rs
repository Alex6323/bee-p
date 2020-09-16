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

pub use crate::atomic::payload::signed_transaction::{input::Input, output::Output};
use crate::atomic::payload::Payload;

use serde::{ser::SerializeStruct, Serialize, Serializer};

#[derive(Clone)]
pub struct UnsignedTransaction {
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub payload: Option<Vec<Payload>>,
}


impl Serialize for UnsignedTransaction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serializer = serializer.serialize_struct("UnsignedTransaction", 6)?;
        serializer.serialize_field("Transaction Type", &0u8)?;
        serializer.serialize_field("Input Count", &(self.inputs.len() as u8))?;
        serializer.serialize_field("Inputs", self.inputs.as_slice())?;
        serializer.serialize_field("Output Count", &(self.outputs.len() as u8))?;
        serializer.serialize_field("Outputs", &self.outputs[0])?;
        serializer.serialize_field("Payload Length", &0u8)?;
        serializer.serialize_field("Payload", &Option::<Vec<Input>>::None)?;
        serializer.end()
    }
}

