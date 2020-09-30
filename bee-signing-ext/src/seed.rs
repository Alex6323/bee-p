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

//! The eed type that integrate both binary and ternary seed.

use crate::binary::{Ed25519Seed, Error};

#[cfg(feature = "std")]
use bee_ternary::{T1B1Buf, T5B1Buf, TritBuf};

/// The general Iota seed
pub enum Seed {
    /// Ed25519 variant
    Ed25519(Ed25519Seed),
    #[cfg(feature = "std")]
    /// Wots variant
    Wots(bee_signing::ternary::seed::Seed),
}

impl Seed {
    /// Create seed from ed25519 bytes
    pub fn from_ed25519_bytes(bytes: &[u8]) -> Result<Self, Error> {
        Ok(Seed::Ed25519(Ed25519Seed::from_bytes(bytes)?))
    }

    /// Create seed from wots trits
    #[cfg(feature = "std")]
    pub fn from_wots_tritbuf(trits: &TritBuf<T5B1Buf>) -> Result<Self, Error> {
        if trits.as_i8_slice().len() != 49 {
            return Err(Error::InvalidLength(49));
        }
        let trits: TritBuf<T1B1Buf> = trits.encode();
        Ok(Seed::Wots(
            bee_signing::ternary::seed::Seed::from_trits(trits).map_err(|_| Error::ConvertError)?,
        ))
    }
}
