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

pub(crate) mod key_range;

use bee_common_ext::packable::{Packable, Read, Write};
use bee_message::MessageId;

use serde::Deserialize;

use std::ops::{Add, Deref};

/// A wrapper around a `u32` that represents a milestone index.
#[derive(Debug, Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize)]
pub struct MilestoneIndex(pub u32);

impl Deref for MilestoneIndex {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u32> for MilestoneIndex {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl Add for MilestoneIndex {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(*self + *other)
    }
}

impl Packable for MilestoneIndex {
    type Error = std::io::Error;

    fn packed_len(&self) -> usize {
        self.0.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self(u32::unpack(reader)?))
    }
}

#[derive(Clone)]
pub struct Milestone {
    pub(crate) message_id: MessageId,
    pub(crate) index: MilestoneIndex,
}

impl Milestone {
    pub fn new(message_id: MessageId, index: MilestoneIndex) -> Self {
        Self { message_id, index }
    }

    pub fn message_id(&self) -> &MessageId {
        &self.message_id
    }

    pub fn index(&self) -> MilestoneIndex {
        self.index
    }
}
