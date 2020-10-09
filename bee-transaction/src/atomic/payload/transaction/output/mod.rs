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

use serde::{Deserialize, Serialize};

use super::super::WriteBytes;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Output {
    SignatureLockedSingle(SignatureLockedSingleOutput),
}

impl From<SignatureLockedSingleOutput> for Output {
    fn from(output: SignatureLockedSingleOutput) -> Self {
        Self::SignatureLockedSingle(output)
    }
}

impl WriteBytes for Output {
    fn len_bytes(&self) -> usize {
        match self {
            Self::SignatureLockedSingle(output) => 0u8.len_bytes() + output.len_bytes(),
        }
    }
    fn write_bytes(&self, buffer: &mut Vec<u8>) {
        match self {
            Self::SignatureLockedSingle(output) => {
                0u8.write_bytes(buffer);
                output.write_bytes(buffer);
            }
        }
    }
}
