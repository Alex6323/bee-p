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
    flags: Flags,
    milestone_index: MilestoneIndex,
    arrival_timestamp: u64,
    solidification_timestamp: u64,
    confirmation_timestamp: u64,
    cone_index: Option<MilestoneIndex>, /* maybe merge milestone_index and cone_index; keep it like that for now to
                                         * avoid conflicts; */
    otrsi: Option<MilestoneIndex>,
    ytrsi: Option<MilestoneIndex>,
}

impl TransactionMetadata {
    pub fn new(
        flags: Flags,
        milestone_index: MilestoneIndex,
        arrival_timestamp: u64,
        solidification_timestamp: u64,
        confirmation_timestamp: u64,
        cone_index: Option<MilestoneIndex>,
        otrsi: Option<MilestoneIndex>,
        ytrsi: Option<MilestoneIndex>,
    ) -> Self {
        Self {
            flags,
            milestone_index,
            arrival_timestamp,
            solidification_timestamp,
            confirmation_timestamp,
            cone_index,
            otrsi,
            ytrsi,
        }
    }

    pub fn arrived() -> Self {
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

    pub fn arrival_timestamp(&self) -> u64 {
        self.arrival_timestamp
    }

    pub fn solidification_timestamp(&self) -> u64 {
        self.solidification_timestamp
    }

    pub fn set_solidification_timestamp(&mut self, timestamp: u64) {
        self.solidification_timestamp = timestamp;
    }

    pub fn cone_index(&self) -> Option<MilestoneIndex> {
        self.cone_index
    }

    pub fn set_cone_index(&mut self, cone_index: MilestoneIndex) {
        self.cone_index = Some(cone_index);
    }

    pub fn otrsi(&self) -> Option<MilestoneIndex> {
        self.otrsi
    }

    pub fn set_otrsi(&mut self, otrsi: MilestoneIndex) {
        self.otrsi = Some(otrsi);
    }

    pub fn ytrsi(&self) -> Option<MilestoneIndex> {
        self.ytrsi
    }

    pub fn set_ytrsi(&mut self, ytrsi: MilestoneIndex) {
        self.ytrsi = Some(ytrsi);
    }

    pub fn confirmation_timestamp(&self) -> u64 {
        self.confirmation_timestamp
    }

    pub fn set_confirmation_timestamp(&mut self, timestamp: u64) {
        self.confirmation_timestamp = timestamp;
    }

    pub fn solidify(&mut self) {
        self.flags.set_solid(true);
        self.solidification_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Clock may have gone backwards")
            .as_millis() as u64;
    }

    pub fn confirm(&mut self) {
        self.flags.set_confirmed(true);
        self.confirmation_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Clock may have gone backwards")
            .as_millis() as u64;
    }
}
