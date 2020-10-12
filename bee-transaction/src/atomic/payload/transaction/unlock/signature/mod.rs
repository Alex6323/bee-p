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

use crate::atomic::packable::{Error as PackableError, Packable, Read, Write};

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
    fn packed_len(&self) -> usize {
        0u8.packed_len()
            + match self {
                Self::Wots(signature) => signature.packed_len(),
                Self::Ed25519(signature) => signature.packed_len(),
            }
    }

    fn pack<W: Write>(&self, buf: &mut W) -> Result<(), PackableError> {
        match self {
            Self::Wots(signature) => {
                0u8.pack(buf)?;
                signature.pack(buf)?;
            }
            Self::Ed25519(signature) => {
                1u8.pack(buf)?;
                signature.pack(buf)?;
            }
        }

        Ok(())
    }

    fn unpack<R: Read>(buf: &mut R) -> Result<Self, PackableError>
    where
        Self: Sized,
    {
        Ok(match u8::unpack(buf)? {
            0 => Self::Wots(WotsSignature::unpack(buf)?),
            1 => Self::Ed25519(Ed25519Signature::unpack(buf)?),
            _ => unreachable!(),
        })
    }
}
