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

mod ed25519;
mod wots;

pub use ed25519::Ed25519Signature;
pub use wots::WotsSignature;

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum SignatureUnlock {
    Wots(WotsSignature),
    Ed25519(Ed25519Signature),
}

impl From<WotsSignature> for SignatureUnlock {
    fn from(signature: WotsSignature) -> Self {
        Self::Wots(signature)
    }
}

impl From<Ed25519Signature> for SignatureUnlock {
    fn from(signature: Ed25519Signature) -> Self {
        Self::Ed25519(signature)
    }
}

impl Packable for SignatureUnlock {
    type Error = Error;

    fn packed_len(&self) -> usize {
        match self {
            Self::Wots(signature) => 0u8.packed_len() + signature.packed_len(),
            Self::Ed25519(signature) => 1u8.packed_len() + signature.packed_len(),
        }
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            Self::Wots(signature) => {
                0u8.pack(writer)?;
                signature.pack(writer)?;
            }
            Self::Ed25519(signature) => {
                1u8.pack(writer)?;
                signature.pack(writer)?;
            }
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(match u8::unpack(reader)? {
            0 => Self::Wots(WotsSignature::unpack(reader)?),
            1 => Self::Ed25519(Ed25519Signature::unpack(reader)?),
            _ => return Err(Self::Error::InvalidVariant),
        })
    }
}
