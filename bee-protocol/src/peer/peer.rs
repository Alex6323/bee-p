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
