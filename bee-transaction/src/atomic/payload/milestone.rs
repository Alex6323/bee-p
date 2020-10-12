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

use crate::atomic::packable::{Error as PackableError, Packable, Read, Write};

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
    fn packed_len(&self) -> usize {
        self.index.packed_len() + self.timestamp.packed_len() + 64 + 64 * self.signatures.len()
    }

    fn pack<W: Write>(&self, buf: &mut W) -> Result<(), PackableError> {
        self.index.pack(buf)?;

        self.timestamp.pack(buf)?;

        buf.write(self.merkle_proof.as_ref())?;

        (self.signatures.len() as u32).pack(buf)?;

        for signature in &self.signatures {
            buf.write(signature.as_ref())?;
        }

        Ok(())
    }

    fn unpack<R: Read>(buf: &mut R) -> Result<Self, PackableError>
    where
        Self: Sized,
    {
        let index = u32::unpack(buf)?;

        let timestamp = u64::unpack(buf)?;

        let mut merkle_proof = [0u8; 64];
        buf.read(&mut merkle_proof)?;
        let merkle_proof = Box::new(merkle_proof);

        let mut signatures = vec![];
        let signatures_len = u32::unpack(buf)?;

        for _ in 0..signatures_len {
            let mut signature = vec![0u8; 64];
            buf.read(&mut signature)?;
            signatures.push(signature.into_boxed_slice());
        }

        Ok(Self {
            index,
            timestamp,
            merkle_proof,
            signatures,
        })
    }
}
