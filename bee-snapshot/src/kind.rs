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

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Kind {
    /// Full is a local snapshot which contains the full ledger entry for a given milestone plus the milestone diffs
    /// which subtracted to the ledger milestone reduce to the snapshot milestone ledger.
    Full = 0,
    /// Delta is a local snapshot which contains solely diffs of milestones newer than a certain ledger milestone
    /// instead of the complete ledger state of a given milestone.
    Delta = 1,
}

impl Packable for Kind {
    fn packed_len(&self) -> usize {
        0u8.packed_len()
    }

    fn pack<W: Write>(&self, buf: &mut W) -> Result<(), PackableError> {
        (*self as u8).pack(buf)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(buf: &mut R) -> Result<Self, PackableError>
    where
        Self: Sized,
    {
        Ok(match u8::unpack(buf)? {
            0 => Kind::Full,
            1 => Kind::Delta,
            _ => return Err(PackableError::InvalidVariant),
        })
    }
}
