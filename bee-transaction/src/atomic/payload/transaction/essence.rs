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

pub use crate::atomic::payload::transaction::{input::Input, output::Output};
use crate::atomic::{payload::Payload, Error};

use serde::{Deserialize, Serialize};

use alloc::vec::Vec;

use super::super::WriteBytes;

// TODO remove pub(crate)
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TransactionEssence {
    pub(crate) inputs: Box<[Input]>,
    pub(crate) outputs: Box<[Output]>,
    pub(crate) payload: Option<Payload>,
}

impl TransactionEssence {
    pub fn builder() -> TransactionEssenceBuilder {
        TransactionEssenceBuilder::new()
    }

    pub fn inputs(&self) -> &[Input] {
        &self.inputs
    }

    pub fn outputs(&self) -> &[Output] {
        &self.outputs
    }

    pub fn payload(&self) -> &Option<Payload> {
        &self.payload
    }
}

impl WriteBytes for TransactionEssence {
    fn len_bytes(&self) -> usize {
        0u8.len_bytes()
            + 0u16.len_bytes()
            + self.inputs.iter().map(|input| input.len_bytes()).sum::<usize>()
            + 0u16.len_bytes()
            + self.outputs.iter().map(|output| output.len_bytes()).sum::<usize>()
            + 0u32.len_bytes()
            + self.payload.iter().map(|payload| payload.len_bytes()).sum::<usize>()
    }

    fn write_bytes(&self, buffer: &mut Vec<u8>) {
        0u8.write_bytes(buffer);

        (self.inputs.len() as u16).write_bytes(buffer);
        for input in self.inputs.as_ref() {
            input.write_bytes(buffer);
        }

        (self.outputs.len() as u16).write_bytes(buffer);
        for output in self.outputs.as_ref() {
            output.write_bytes(buffer);
        }

        if let Some(payload) = &self.payload {
            (payload.len_bytes() as u32).write_bytes(buffer);
            payload.write_bytes(buffer);
        } else {
            0u32.write_bytes(buffer);
        }
    }
}

#[derive(Debug, Default)]
pub struct TransactionEssenceBuilder {
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    payload: Option<Payload>,
}

impl TransactionEssenceBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_input(mut self, input: Input) -> Self {
        self.inputs.push(input);
        self
    }

    pub fn add_output(mut self, output: Output) -> Self {
        self.outputs.push(output);
        self
    }

    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn finish(self) -> Result<TransactionEssence, Error> {
        if self.inputs.is_empty() {
            return Err(Error::NoInput);
        }

        if self.outputs.is_empty() {
            return Err(Error::NoOutput);
        }

        Ok(TransactionEssence {
            inputs: self.inputs.into_boxed_slice(),
            outputs: self.outputs.into_boxed_slice(),
            payload: self.payload,
        })
    }
}
