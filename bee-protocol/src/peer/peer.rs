// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{milestone::MilestoneIndex, protocol::ProtocolMetrics};

use bee_network::{Address, EndpointId, Origin};

use std::sync::atomic::{AtomicU32, Ordering};

pub struct Peer {
    pub(crate) epid: EndpointId,
    pub(crate) address: Address,
    pub(crate) origin: Origin,
    pub(crate) metrics: ProtocolMetrics,
    pub(crate) solid_milestone_index: AtomicU32,
    pub(crate) snapshot_milestone_index: AtomicU32,
}

impl Peer {
    pub fn new(epid: EndpointId, address: Address, origin: Origin) -> Self {
        Self {
            epid,
            address,
            origin,
            metrics: ProtocolMetrics::default(),
            solid_milestone_index: AtomicU32::new(0),
            snapshot_milestone_index: AtomicU32::new(0),
        }
    }

    pub(crate) fn set_solid_milestone_index(&self, index: MilestoneIndex) {
        self.solid_milestone_index.store(index, Ordering::Relaxed);
    }

    pub(crate) fn solid_milestone_index(&self) -> MilestoneIndex {
        self.solid_milestone_index.load(Ordering::Relaxed)
    }

    pub(crate) fn set_snapshot_milestone_index(&self, index: MilestoneIndex) {
        self.snapshot_milestone_index.store(index, Ordering::Relaxed);
    }

    pub(crate) fn snapshot_milestone_index(&self) -> MilestoneIndex {
        self.snapshot_milestone_index.load(Ordering::Relaxed)
    }
}
