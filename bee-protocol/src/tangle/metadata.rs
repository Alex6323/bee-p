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

use bee_storage::persistable::Persistable;

use std::time::{SystemTime, UNIX_EPOCH};

// TODO Should it really be copy ?
#[derive(Copy, Clone, Default)]
pub struct TransactionMetadata {
    pub(crate) flags: Flags,
    pub(crate) milestone_index: MilestoneIndex,
    pub(crate) arrival_timestamp: u64,
    pub(crate) solidification_timestamp: u64,
    pub(crate) confirmation_timestamp: u64,
    // maybe merge milestone_index and cone_index; keep it like that in the mean time to avoid conflicts;
    pub(crate) cone_index: Option<MilestoneIndex>,
    pub(crate) otrsi: Option<MilestoneIndex>,
    pub(crate) ytrsi: Option<MilestoneIndex>,
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
}

impl Persistable for TransactionMetadata {
    fn encode_persistable(&self, buffer: &mut Vec<u8>) {
        // encode struct in order
        // 1- encode flags
        self.flags.bits().encode_persistable(buffer);
        // 2- encode milestone_index
        self.milestone_index.encode_persistable(buffer);
        // 3- encode arrival_timestamp
        self.arrival_timestamp.encode_persistable(buffer);
        // 4- encode solidification_timestamp
        self.solidification_timestamp.encode_persistable(buffer);
        // 5- encode confirmation_timestamp
        self.confirmation_timestamp.encode_persistable(buffer);
    }
    fn decode_persistable(slice: &[u8]) -> Self {
        // decode struct in order
        // 1- decode flags
        let flags = Flags::from_bits(u8::decode_persistable(&slice[0..1])).unwrap();
        // 2- decode milestone_index
        let milestone_index = MilestoneIndex::decode_persistable(&slice[1..5]);
        // 3- decode arrival_timestamp
        let arrival_timestamp = u64::decode_persistable(&slice[5..13]);
        // 4- decode solidification_timestamp
        let solidification_timestamp = u64::decode_persistable(&slice[13..21]);
        // 5- decode confirmation_timestamp
        let confirmation_timestamp = u64::decode_persistable(&slice[21..29]);
        Self {
            flags,
            milestone_index,
            arrival_timestamp,
            solidification_timestamp,
            confirmation_timestamp,
            cone_index: None,
            otrsi: None,
            ytrsi: None
        }
    }
}
