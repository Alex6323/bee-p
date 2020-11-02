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

use crate::Error;

use bee_common::packable::{Packable, Read, Write};
use bee_message::{
    payload::transaction::{SignatureLockedSingleOutput, UTXOInput},
    MessageId,
};

pub(crate) struct Output {
    message_id: MessageId,
    output_id: UTXOInput,
    output: SignatureLockedSingleOutput,
}

impl Packable for Output {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.message_id.packed_len() + self.output_id.packed_len() + self.output.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.message_id.pack(writer)?;
        self.output_id.pack(writer)?;
        self.output.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self {
            message_id: MessageId::unpack(reader)?,
            output_id: UTXOInput::unpack(reader)?,
            output: SignatureLockedSingleOutput::unpack(reader)?,
        })
    }
}
