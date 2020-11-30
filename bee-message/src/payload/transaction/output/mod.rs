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

mod address;
mod output_id;
mod signature_locked_single;

pub use address::{Address, Ed25519Address, WotsAddress, ED25519_ADDRESS_LENGTH};
pub use output_id::{OutputId, OUTPUT_ID_LENGTH};
pub use signature_locked_single::SignatureLockedSingleOutput;

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, Ord, PartialOrd)]
#[serde(tag = "type", content = "data")]
pub enum Output {
    SignatureLockedSingle(SignatureLockedSingleOutput),
}

impl From<SignatureLockedSingleOutput> for Output {
    fn from(output: SignatureLockedSingleOutput) -> Self {
        Self::SignatureLockedSingle(output)
    }
}

impl Packable for Output {
    type Error = Error;

    fn packed_len(&self) -> usize {
        match self {
            Self::SignatureLockedSingle(output) => 0u8.packed_len() + output.packed_len(),
        }
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            Self::SignatureLockedSingle(output) => {
                0u8.pack(writer)?;
                output.pack(writer)?;
            }
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(match u8::unpack(reader)? {
            0 => Self::SignatureLockedSingle(SignatureLockedSingleOutput::unpack(reader)?),
            _ => return Err(Self::Error::InvalidVariant),
        })
    }
}
