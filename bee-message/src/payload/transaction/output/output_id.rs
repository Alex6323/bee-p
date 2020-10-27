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

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, Hash)]
pub struct OutputId {
    transaction_id: TransactionId,
    index: u16,
}

impl OutputId {
    pub fn new(transaction_id: TransactionId, index: u16) -> Result<Self, Error> {
        if !INPUT_OUTPUT_INDEX_RANGE.contains(&index) {
            return Err(Error::InvalidIndex);
        }

        Ok(Self { transaction_id, index })
    }

    pub fn transaction_id(&self) -> &TransactionId {
        &self.transaction_id
    }

    pub fn index(&self) -> u16 {
        self.index
    }
}

impl core::fmt::Display for OutputId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "{}{}",
            self.transaction_id.to_string(),
            hex::encode(self.index.to_le_bytes())
        )
    }
}

impl Packable for OutputId {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.transaction_id.packed_len() + self.index.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.transaction_id.pack(writer)?;
        self.index.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let transaction_id = TransactionId::unpack(reader)?;
        let index = u16::unpack(reader)?;

        Ok(Self::new(transaction_id, index).map_err(|_| Self::Error::InvalidSyntax)?)
    }
}
