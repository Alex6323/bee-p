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

use crate::{
    payload::transaction::{output::OutputId, TransactionId},
    Error,
};

use bee_common::packable::{Packable, Read, Write};

use serde::{Deserialize, Serialize};

use core::convert::{From, TryFrom};

#[derive(Clone, Eq, PartialEq, Deserialize, Serialize, Hash, Ord, PartialOrd)]
pub struct UTXOInput(OutputId);

impl From<OutputId> for UTXOInput {
    fn from(id: OutputId) -> Self {
        UTXOInput(id)
    }
}

impl TryFrom<&str> for UTXOInput {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(UTXOInput(OutputId::try_from(value)?))
    }
}

impl UTXOInput {
    pub fn new(id: TransactionId, index: u16) -> Result<Self, Error> {
        Ok(Self(OutputId::new(id, index)?))
    }

    pub fn output_id(&self) -> &OutputId {
        &self.0
    }
}

impl core::fmt::Display for UTXOInput {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl core::fmt::Debug for UTXOInput {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "UTXOInput({})", self.0)
    }
}

impl Packable for UTXOInput {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.0.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self(OutputId::unpack(reader)?))
    }
}
