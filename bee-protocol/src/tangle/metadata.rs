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

use crate::{milestone::MilestoneIndex, tangle::Flags};

use std::time::{SystemTime, UNIX_EPOCH};

// TODO Should it really be copy ?
#[derive(Copy, Clone, Default)]
pub struct TransactionMetadata {
    pub flags: Flags,
    pub milestone_index: MilestoneIndex,
    pub arrival_timestamp: u64,
    pub solidification_timestamp: u64,
    pub confirmation_timestamp: u64,
}

impl TransactionMetadata {
    pub fn new() -> Self {
        Self {
            arrival_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Clock may have gone backwards")
                .as_millis() as u64,
            ..Self::default()
        }
    }

    pub fn flags(&self) -> &Flags {
        &self.flags
    }

    pub fn flags_mut(&mut self) -> &mut Flags {
        &mut self.flags
    }

    pub fn milestone_index(&self) -> MilestoneIndex {
        self.milestone_index
    }

    pub fn set_milestone_index(&mut self, index: MilestoneIndex) {
        self.milestone_index = index;
    }

    pub fn set_confirmation_timestamp(&mut self, timestamp: u64) {
        self.confirmation_timestamp = timestamp;
    }
    pub fn confirmation_timestamp(&self) -> u64 {
        self.confirmation_timestamp
    }
    pub fn arrival_timestamp(&self) -> u64 {
        self.arrival_timestamp
    }
    pub fn solidification_timestamp(&self) -> u64 {
        self.solidification_timestamp
    }
}
