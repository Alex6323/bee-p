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
    pub(crate) flags: Flags,
    pub(crate) arrival_timestamp: u64,
    pub(crate) solidification_timestamp: u64,
    pub(crate) milestone_index: MilestoneIndex,
    // maybe merge milestone_index and cone_index; keep it like that in the mean time to avoid conflicts;
    pub(crate) cone_index: Option<MilestoneIndex>,
    pub(crate) otrsi: Option<MilestoneIndex>,
    pub(crate) ytrsi: Option<MilestoneIndex>,
    pub(crate) num_selected: u8,
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

    pub fn milestone_index(&self) -> MilestoneIndex {
        self.milestone_index
    }

    pub fn is_confirmed(&self) -> bool {
        self.flags.is_confirmed()
    }
}
