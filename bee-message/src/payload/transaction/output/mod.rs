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
mod signature_locked_single;

pub use address::{Address, Ed25519Address, WotsAddress};
pub use signature_locked_single::SignatureLockedSingleOutput;

use bee_common_ext::packable::{Error as PackableError, Packable, Read, Write};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Output {
    SignatureLockedSingle(SignatureLockedSingleOutput),
}

impl From<SignatureLockedSingleOutput> for Output {
    fn from(output: SignatureLockedSingleOutput) -> Self {
        Self::SignatureLockedSingle(output)
    }
}

impl Packable for Output {
    fn packed_len(&self) -> usize {
        match self {
            Self::SignatureLockedSingle(output) => 0u8.packed_len() + output.packed_len(),
        }
    }

    fn pack<W: Write>(&self, buf: &mut W) -> Result<(), PackableError> {
        match self {
            Self::SignatureLockedSingle(output) => {
                0u8.pack(buf)?;
                output.pack(buf)?;
            }
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(buf: &mut R) -> Result<Self, PackableError>
    where
        Self: Sized,
    {
        Ok(match u8::unpack(buf)? {
            0 => Self::SignatureLockedSingle(SignatureLockedSingleOutput::unpack(buf)?),
            _ => return Err(PackableError::InvalidVariant),
        })
    }
}
