use crate::message::Heartbeat;
use crate::peer::PeerMetrics;
use bee_network::EndpointId;

pub struct Peer {
    pub(crate) epid: EndpointId,
    pub(crate) metrics: PeerMetrics,
    heartbeat: Heartbeat,
}

impl Peer {
    pub fn new(epid: EndpointId) -> Self {
        Self {
            epid: epid,
            metrics: PeerMetrics::default(),
            heartbeat: Heartbeat::default(),
        }
    }
}
