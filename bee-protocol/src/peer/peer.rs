use crate::{
    message::Heartbeat,
    peer::PeerMetrics,
};
use bee_network::{
    EndpointId,
    Role,
};

pub struct Peer {
    pub(crate) epid: EndpointId,
    pub(crate) role: Role,
    pub(crate) metrics: PeerMetrics,
    heartbeat: Heartbeat,
}

impl Peer {
    pub fn new(epid: EndpointId, role: Role) -> Self {
        Self {
            epid,
            role,
            metrics: PeerMetrics::default(),
            heartbeat: Heartbeat::default(),
        }
    }
}
