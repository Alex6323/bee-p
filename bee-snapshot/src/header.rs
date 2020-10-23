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

use crate::kind::Kind;

use bee_common_ext::packable::{Error as PackableError, Packable, Read, Write};
use bee_message::MessageId;

const SNAPSHOT_VERSION: u8 = 1;

pub struct SnapshotHeader {
    pub(crate) kind: Kind,
    pub(crate) timestamp: u64,
    // TODO replace with ED25 pk
    pub(crate) coordinator: [u8; 32],
    pub(crate) sep_index: u32,
    pub(crate) sep_id: MessageId,
    pub(crate) ledger_index: u32,
    pub(crate) ledger_id: MessageId,
}

impl SnapshotHeader {
    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn coordinator(&self) -> &[u8; 32] {
        &self.coordinator
    }

    pub fn sep_index(&self) -> u32 {
        self.sep_index
    }

    pub fn sep_id(&self) -> &MessageId {
        &self.sep_id
    }

    pub fn ledger_index(&self) -> u32 {
        self.ledger_index
    }

    pub fn ledger_id(&self) -> &MessageId {
        &self.ledger_id
    }
}

impl Packable for SnapshotHeader {
    fn packed_len(&self) -> usize {
        SNAPSHOT_VERSION.packed_len()
            + self.kind.packed_len()
            + self.timestamp.packed_len()
            // TODO impl packable for byte slices or ED25 packable
        +32+self.sep_index.packed_len()+self.sep_id.packed_len()+self.ledger_index.packed_len()+self.ledger_id.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), PackableError> {
        SNAPSHOT_VERSION.pack(writer)?;
        self.kind.pack(writer)?;
        self.timestamp.pack(writer)?;
        // TODO packable on bytes
        writer.write_all(&self.coordinator)?;
        // self.coordinator.pack(writer)?;
        self.sep_index.pack(writer)?;
        self.sep_id.pack(writer)?;
        self.ledger_index.pack(writer)?;
        self.ledger_id.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, PackableError>
    where
        Self: Sized,
    {
        let version = u8::unpack(reader)?;

        if version != SNAPSHOT_VERSION {
            return Err(PackableError::InvalidVersion(SNAPSHOT_VERSION, version));
        }

        let kind = Kind::unpack(reader)?;
        let timestamp = u64::unpack(reader)?;
        // TODO pk type
        let mut coordinator = [0u8; 32];
        reader.read_exact(&mut coordinator)?;
        let sep_index = u32::unpack(reader)?;
        let sep_id = MessageId::unpack(reader)?;
        let ledger_index = u32::unpack(reader)?;
        let ledger_id = MessageId::unpack(reader)?;

        Ok(Self {
            kind,
            timestamp,
            coordinator,
            sep_index,
            sep_id,
            ledger_index,
            ledger_id,
        })
    }
}
