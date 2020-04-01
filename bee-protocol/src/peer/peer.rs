use crate::{
    message::Heartbeat,
    peer::PeerMetrics,
};
use bee_network::{
    Address,
    EndpointId,
    Origin,
};

pub struct Peer {
    pub(crate) epid: EndpointId,
    pub(crate) address: Address,
    pub(crate) origin: Origin,
    pub(crate) metrics: PeerMetrics,
    pub(crate) heartbeat: Heartbeat,
}

impl Peer {
    pub fn new(epid: EndpointId, address: Address, origin: Origin) -> Self {
        Self {
            epid,
            address,
            origin,
            metrics: PeerMetrics::default(),
            heartbeat: Heartbeat::default(),
        }
    }
}
