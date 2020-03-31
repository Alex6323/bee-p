use crate::{
    message::Heartbeat,
    peer::PeerMetrics,
};
use bee_network::{
    EndpointId,
    Origin,
};

pub struct Peer {
    pub(crate) epid: EndpointId,
    pub(crate) origin: Origin,
    pub(crate) metrics: PeerMetrics,
    pub(crate) heartbeat: Heartbeat,
}

impl Peer {
    pub fn new(epid: EndpointId, origin: Origin) -> Self {
        Self {
            epid,
            origin,
            metrics: PeerMetrics::default(),
            heartbeat: Heartbeat::default(),
        }
    }
}
