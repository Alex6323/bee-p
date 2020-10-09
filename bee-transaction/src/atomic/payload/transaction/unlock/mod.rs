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

mod reference;
mod signature;

pub use reference::ReferenceUnlock;
pub use signature::{Ed25519Signature, SignatureUnlock, WotsSignature};

use crate::atomic::packable::{Buf, BufMut, Packable};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum UnlockBlock {
    Reference(ReferenceUnlock),
    Signature(SignatureUnlock),
}

impl From<ReferenceUnlock> for UnlockBlock {
    fn from(reference: ReferenceUnlock) -> Self {
        Self::Reference(reference)
    }
}

impl From<SignatureUnlock> for UnlockBlock {
    fn from(signature: SignatureUnlock) -> Self {
        Self::Signature(signature)
    }
}

impl Packable for UnlockBlock {
    fn len_bytes(&self) -> usize {
        0u8.len_bytes()
            + match self {
                Self::Reference(reference) => reference.len_bytes(),
                Self::Signature(signature) => signature.len_bytes(),
            }
    }

    fn pack_bytes<B: BufMut>(&self, buffer: &mut B) {
        match self {
            Self::Reference(reference) => {
                0u8.pack_bytes(buffer);
                reference.pack_bytes(buffer);
            }
            Self::Signature(signature) => {
                0u8.pack_bytes(buffer);
                signature.pack_bytes(buffer);
            }
        }
    }

    fn unpack_bytes<B: Buf>(buffer: &mut B) -> Self {
        match u8::unpack_bytes(buffer) {
            0 => Self::Reference(ReferenceUnlock::unpack_bytes(buffer)),
            1 => Self::Signature(SignatureUnlock::unpack_bytes(buffer)),
            _ => unreachable!(),
        }
    }
}
