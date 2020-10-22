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

use crate::{output::Output, spent::Spent};

use bee_common_ext::packable::{Error as PackableError, Packable, Read, Write};

pub(crate) struct MilestoneDiff {
    index: u32,
    created: Vec<Output>,
    consumed: Vec<Spent>,
}

impl Packable for MilestoneDiff {
    fn packed_len(&self) -> usize {
        // TODO finish
        self.index.packed_len() + 0u64.packed_len() + 0u64.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), PackableError> {
        self.index.pack(writer)?;
        (self.created.len() as u64).pack(writer)?;
        for output in self.created.iter() {
            output.pack(writer)?;
        }
        (self.consumed.len() as u64).pack(writer)?;
        for spent in self.consumed.iter() {
            spent.pack(writer)?;
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, PackableError>
    where
        Self: Sized,
    {
        let index = u32::unpack(reader)?;
        let created_count = u64::unpack(reader)? as usize;
        let mut created = Vec::with_capacity(created_count);
        for _ in 0..created_count {
            created.push(Output::unpack(reader)?);
        }
        let consumed_count = u64::unpack(reader)? as usize;
        let mut consumed = Vec::with_capacity(consumed_count);
        for _ in 0..consumed_count {
            consumed.push(Spent::unpack(reader)?);
        }

        Ok(Self {
            index,
            created,
            consumed,
        })
    }
}
