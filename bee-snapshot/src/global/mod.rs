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

mod config;
mod file;

pub use config::{GlobalSnapshotConfig, GlobalSnapshotConfigBuilder};
pub use file::Error as FileError;

use bee_ledger::state::LedgerState;
use bee_protocol::MilestoneIndex;

pub struct GlobalSnapshot {
    index: MilestoneIndex,
    state: LedgerState,
}

impl GlobalSnapshot {
    pub fn index(&self) -> &MilestoneIndex {
        &self.index
    }

    pub fn state(&self) -> &LedgerState {
        &self.state
    }

    pub fn into_state(self) -> LedgerState {
        self.state
    }
}
