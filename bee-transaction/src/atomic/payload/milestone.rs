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

use crate::atomic::packable::{Buf, BufMut, Packable};

use serde::{Deserialize, Serialize};

use alloc::{boxed::Box, vec::Vec};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Milestone {
    index: u32,
    timestamp: u64,
    // TODO length is 64, change to array when std::array::LengthAtMost32 disappears.
    merkle_proof: Box<[u8]>,
    // TODO length is 64, change to array when std::array::LengthAtMost32 disappears.
    signatures: Vec<Box<[u8]>>,
}

impl Milestone {
    pub fn new(index: u32, timestamp: u64, merkle_proof: Box<[u8]>, signatures: Vec<Box<[u8]>>) -> Self {
        Self {
            index,
            timestamp,
            merkle_proof,
            signatures,
        }
    }
}

impl Packable for Milestone {
    fn len_bytes(&self) -> usize {
        self.index.len_bytes() + self.timestamp.len_bytes() + 64 + 64 * self.signatures.len()
    }

    fn pack_bytes<B: BufMut>(&self, buffer: &mut B) {
        self.index.pack_bytes(buffer);

        self.timestamp.pack_bytes(buffer);

        Self::pack_slice(self.merkle_proof.as_ref(), buffer);

        (self.signatures.len() as u32).pack_bytes(buffer);

        for signature in &self.signatures {
            Self::pack_slice(signature.as_ref(), buffer);
        }
    }

    fn unpack_bytes<B: Buf>(buffer: &mut B) -> Self {
        let index = u32::unpack_bytes(buffer);

        let timestamp = u64::unpack_bytes(buffer);

        let merkle_proof = Self::unpack_vec(buffer, 64).into_boxed_slice();

        let mut signatures = vec![];
        let signatures_len = u32::unpack_bytes(buffer);

        for _ in 0..signatures_len {
            let signature = Self::unpack_vec(buffer, 64).into_boxed_slice();
            signatures.push(signature);
        }

        Self {
            index,
            timestamp,
            merkle_proof,
            signatures,
        }
    }
}
