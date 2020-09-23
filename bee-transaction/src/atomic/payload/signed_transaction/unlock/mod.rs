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

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum UnlockBlock {
    Reference(ReferenceUnlock),
    Signature(SignatureUnlock),
}

impl UnlockBlock {
    /// Create UnlockBLock from Ed25519Signature.
    pub fn from_ed25519_signature(signature: Ed25519Signature) -> Self {
        Self::Signature(SignatureUnlock::Ed25519(signature))
    }

    /// Create UnlockBLock from WotsSignature.
    pub fn from_wots_signature(signature: WotsSignature) -> Self {
        Self::Signature(SignatureUnlock::Wots(signature))
    }

    /// Create UnlockBlock from reference index.
    pub fn from_reference_unlock(index: u8) -> Self {
        Self::Reference(ReferenceUnlock { index })
    }
}
