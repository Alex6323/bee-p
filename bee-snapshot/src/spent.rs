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

use crate::{output::Output, Error};

use bee_common::packable::{Packable, Read, Write};
use bee_message::payload::transaction::TransactionId;

pub(crate) struct Spent {
    output: Output,
    transaction_id: TransactionId,
}

impl Packable for Spent {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.output.packed_len() + self.transaction_id.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.output.pack(writer)?;
        self.transaction_id.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self {
            output: Output::unpack(reader)?,
            transaction_id: TransactionId::unpack(reader)?,
        })
    }
}
