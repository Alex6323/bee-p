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

use bee_common_ext::packable::{Error as PackableError, Packable, Read, Write};
use bee_message::prelude::MessageId;

const SNAPSHOT_VERSION: u8 = 1;
const SNAPSHOT_TYPE: u8 = 0;

pub struct SnapshotHeader {
    pub(crate) timestamp: u64,
    // TODO replace with ED25 pk
    pub(crate) coordinator: [u8; 32],
    pub(crate) sep_index: u32,
    pub(crate) sep_id: MessageId,
    pub(crate) ledger_index: u32,
    pub(crate) ledger_id: MessageId,
}

impl SnapshotHeader {
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
            + SNAPSHOT_TYPE.packed_len()
            + self.timestamp.packed_len()
            // TODO impl packable for byte slices
        +32+self.sep_index.packed_len()+self.sep_id.packed_len()+self.ledger_index.packed_len()+self.ledger_id.packed_len()
    }

    fn pack<W: Write>(&self, buf: &mut W) -> Result<(), PackableError> {
        SNAPSHOT_VERSION.pack(buf)?;
        SNAPSHOT_TYPE.pack(buf)?;
        self.timestamp.pack(buf)?;
        // TODO packable on bytes
        buf.write_all(&self.coordinator)?;
        // self.coordinator.pack(buf)?;
        self.sep_index.pack(buf)?;
        self.sep_id.pack(buf)?;
        self.ledger_index.pack(buf)?;
        self.ledger_id.pack(buf)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(buf: &mut R) -> Result<Self, PackableError>
    where
        Self: Sized,
    {
        let _snapshot_version = u8::unpack(buf)?;
        let _snapshot_type = u8::unpack(buf)?;
        // TODO check version and type
        let timestamp = u64::unpack(buf)?;
        // TODO pk type
        let mut coordinator = [0u8; 32];
        buf.read_exact(&mut coordinator)?;
        let sep_index = u32::unpack(buf)?;
        let sep_id = MessageId::unpack(buf)?;
        let ledger_index = u32::unpack(buf)?;
        let ledger_id = MessageId::unpack(buf)?;

        Ok(Self {
            timestamp,
            coordinator,
            sep_index,
            sep_id,
            ledger_index,
            ledger_id,
        })
    }
}
