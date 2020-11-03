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

pub(crate) mod index;
pub(crate) mod key_manager;
pub(crate) mod key_range;

pub use index::MilestoneIndex;

use bee_message::MessageId;

#[derive(Clone)]
pub struct Milestone {
    pub(crate) index: MilestoneIndex,
    pub(crate) message_id: MessageId,
}

impl Milestone {
    pub fn new(index: MilestoneIndex, message_id: MessageId) -> Self {
        Self { index, message_id }
    }

    pub fn index(&self) -> MilestoneIndex {
        self.index
    }

    pub fn message_id(&self) -> &MessageId {
        &self.message_id
    }
}
