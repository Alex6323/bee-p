use crate::protocol::ProtocolMetrics;
use bee_network::{
    Address,
    EndpointId,
    Origin,
};

use std::sync::atomic::AtomicU32;

pub struct Peer {
    pub(crate) epid: EndpointId,
    pub(crate) address: Address,
    pub(crate) origin: Origin,
    pub(crate) metrics: ProtocolMetrics,
    // TODO  MilestoneIndex atomic ?
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
}
