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

use crate::header::SnapshotHeader;

use bee_common_ext::packable::{Error as PackableError, Packable, Read, Write};
use bee_message::prelude::MessageId;

use std::collections::HashSet;

pub struct SnapshotMetadata {
    pub(crate) header: SnapshotHeader,
    pub(crate) solid_entry_points: HashSet<MessageId>,
}

impl SnapshotMetadata {
    pub fn header(&self) -> &SnapshotHeader {
        &self.header
    }

    pub fn solid_entry_points(&self) -> &HashSet<MessageId> {
        &self.solid_entry_points
    }
}

impl Packable for SnapshotMetadata {
    fn packed_len(&self) -> usize {
        self.header.packed_len()
        // + TODO SEP
    }

    fn pack<W: Write>(&self, buf: &mut W) -> Result<(), PackableError> {
        self.header.pack(buf)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(buf: &mut R) -> Result<Self, PackableError>
    where
        Self: Sized,
    {
        // TODO SEP
        Ok(Self {
            header: SnapshotHeader::unpack(buf)?,
            solid_entry_points: HashSet::new(),
        })
    }
}
