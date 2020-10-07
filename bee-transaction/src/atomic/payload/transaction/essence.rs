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

use alloc::vec::Vec;

// TODO remove pub(crate)
#[derive(Debug)]
pub struct TransactionEssence {
    pub(crate) inputs: Vec<Input>,
    pub(crate) outputs: Vec<Output>,
    pub(crate) payload: Option<Payload>,
}

impl TransactionEssence {
    pub fn builder() -> TransactionEssenceBuilder {
        TransactionEssenceBuilder::new()
    }

    pub fn inputs(&self) -> &Vec<Input> {
        &self.inputs
    }

    pub fn outputs(&self) -> &Vec<Output> {
        &self.outputs
    }

    pub fn payload(&self) -> &Option<Payload> {
        &self.payload
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
            inputs: self.inputs,
            outputs: self.outputs,
            payload: self.payload,
        })
    }
}
