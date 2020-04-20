use crate::{
    milestone::MilestoneIndex,
    protocol::ProtocolMetrics,
};

use bee_network::{
    Address,
    EndpointId,
    Origin,
};

use std::sync::atomic::{
    AtomicU32,
    Ordering,
};

pub struct Peer {
    pub(crate) epid: EndpointId,
    pub(crate) address: Address,
    pub(crate) origin: Origin,
    pub(crate) metrics: ProtocolMetrics,
    pub(crate) first_solid_milestone_index: AtomicU32,
    pub(crate) last_solid_milestone_index: AtomicU32,
}

impl Peer {
    pub fn new(epid: EndpointId, address: Address, origin: Origin) -> Self {
        Self {
            epid,
            address,
            origin,
            metrics: ProtocolMetrics::default(),
            first_solid_milestone_index: AtomicU32::new(0),
            last_solid_milestone_index: AtomicU32::new(0),
        }
    }

    pub(crate) fn set_first_solid_milestone_index(&self, index: MilestoneIndex) {
        self.first_solid_milestone_index.store(index, Ordering::Relaxed);
    }

    pub(crate) fn first_solid_milestone_index(&self) -> MilestoneIndex {
        self.first_solid_milestone_index.load(Ordering::Relaxed)
    }

    pub(crate) fn set_last_solid_milestone_index(&self, index: MilestoneIndex) {
        self.last_solid_milestone_index.store(index, Ordering::Relaxed);
    }

    pub(crate) fn last_solid_milestone_index(&self) -> MilestoneIndex {
        self.last_solid_milestone_index.load(Ordering::Relaxed)
    }
}
