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

use bee_message::prelude::MessageId;

use std::ops::{Add, Deref};

/// A wrapper around a `u32` that represents a milestone index.
#[derive(Debug, Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

#[derive(Clone)]
pub struct Milestone {
    pub(crate) hash: MessageId,
    pub(crate) index: MilestoneIndex,
}

impl Milestone {
    pub fn new(hash: MessageId, index: MilestoneIndex) -> Self {
        Self { hash, index }
    }

    pub fn hash(&self) -> &MessageId {
        &self.hash
    }

    pub fn index(&self) -> MilestoneIndex {
        self.index
    }
}
