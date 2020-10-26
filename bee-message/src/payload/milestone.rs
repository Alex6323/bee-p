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

use crate::MessageId;

use bee_common_ext::packable::{Error as PackableError, Packable, Read, Write};

use serde::{Deserialize, Serialize};

use alloc::{boxed::Box, vec::Vec};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Milestone {
    essence: MilestoneEssence,
    // TODO length is 64, change to array when std::array::LengthAtMost32 disappears.
    signatures: Vec<Box<[u8]>>,
}

impl Milestone {
    pub fn new(essence: MilestoneEssence, signatures: Vec<Box<[u8]>>) -> Self {
        Self { essence, signatures }
    }

    pub fn essence(&self) -> &MilestoneEssence {
        &self.essence
    }

    pub fn signatures(&self) -> &Vec<Box<[u8]>> {
        &self.signatures
    }
}

impl Packable for Milestone {
    fn packed_len(&self) -> usize {
        self.essence.packed_len() + 64 * self.signatures.len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), PackableError> {
        self.essence.pack(writer)?;

        (self.signatures.len() as u8).pack(writer)?;
        for signature in &self.signatures {
            writer.write_all(&signature)?;
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, PackableError>
    where
        Self: Sized,
    {
        let essence = MilestoneEssence::unpack(reader)?;

        let signatures_len = u8::unpack(reader)? as usize;
        let mut signatures = Vec::with_capacity(signatures_len);
        for _ in 0..signatures_len {
            let mut signature = vec![0u8; 64];
            reader.read_exact(&mut signature)?;
            signatures.push(signature.into_boxed_slice());
        }

        Ok(Self { essence, signatures })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MilestoneEssence {
    index: u32,
    timestamp: u64,
    parent1: MessageId,
    parent2: MessageId,
    // TODO length is 64, change to array when std::array::LengthAtMost32 disappears.
    merkle_proof: Box<[u8]>,
    public_keys: Vec<[u8; 32]>,
}

impl MilestoneEssence {
    pub fn new(
        index: u32,
        timestamp: u64,
        parent1: MessageId,
        parent2: MessageId,
        merkle_proof: Box<[u8]>,
        public_keys: Vec<[u8; 32]>,
    ) -> Self {
        Self {
            index,
            timestamp,
            parent1,
            parent2,
            merkle_proof,
            public_keys,
        }
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn parent1(&self) -> &MessageId {
        &self.parent1
    }

    pub fn parent2(&self) -> &MessageId {
        &self.parent2
    }

    pub fn merkle_proof(&self) -> &[u8] {
        &self.merkle_proof
    }

    pub fn public_keys(&self) -> &Vec<[u8; 32]> {
        &self.public_keys
    }
}

impl Packable for MilestoneEssence {
    fn packed_len(&self) -> usize {
        self.index.packed_len()
            + self.timestamp.packed_len()
            + self.parent1.packed_len()
            + self.parent2.packed_len()
            + 32
            + 32 * self.public_keys.len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), PackableError> {
        self.index.pack(writer)?;

        self.timestamp.pack(writer)?;

        self.parent1.pack(writer)?;
        self.parent2.pack(writer)?;

        writer.write_all(&self.merkle_proof)?;

        (self.public_keys.len() as u8).pack(writer)?;

        for public_key in &self.public_keys {
            writer.write_all(public_key)?;
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, PackableError>
    where
        Self: Sized,
    {
        let index = u32::unpack(reader)?;

        let timestamp = u64::unpack(reader)?;

        let parent1 = MessageId::unpack(reader)?;
        let parent2 = MessageId::unpack(reader)?;

        let mut merkle_proof = [0u8; 64];
        reader.read_exact(&mut merkle_proof)?;

        let public_keys_len = u8::unpack(reader)? as usize;
        let mut public_keys = Vec::with_capacity(public_keys_len);
        for _ in 0..public_keys_len {
            let mut public_key = [0u8; 32];
            reader.read_exact(&mut public_key)?;
            public_keys.push(public_key);
        }

        Ok(Self {
            index,
            timestamp,
            parent1,
            parent2,
            merkle_proof: Box::new(merkle_proof),
            public_keys,
        })
    }
}
