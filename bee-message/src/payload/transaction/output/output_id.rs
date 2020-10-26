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
    payload::transaction::{constants::INPUT_OUTPUT_INDEX_RANGE, TransactionId},
    Error,
};

use bee_common_ext::packable::{Packable, Read, Write};

use serde::{Deserialize, Serialize};

use alloc::string::ToString;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct OutputId {
    id: TransactionId,
    index: u16,
}

impl OutputId {
    pub fn new(id: TransactionId, index: u16) -> Result<Self, Error> {
        if !INPUT_OUTPUT_INDEX_RANGE.contains(&index) {
            return Err(Error::InvalidIndex);
        }

        Ok(Self { id, index })
    }

    pub fn id(&self) -> &TransactionId {
        &self.id
    }

    pub fn index(&self) -> u16 {
        self.index
    }
}

impl core::fmt::Display for OutputId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}{}", self.id.to_string(), hex::encode(self.index.to_le_bytes()))
    }
}

impl Packable for OutputId {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.id.packed_len() + self.index.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.id.pack(writer)?;
        self.index.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let id = TransactionId::unpack(reader)?;
        let index = u16::unpack(reader)?;

        Ok(Self::new(id, index).map_err(|_| Self::Error::InvalidSyntax)?)
    }
}
