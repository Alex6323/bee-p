use crate::message::Heartbeat;
use crate::peer::PeerMetrics;

pub(crate) struct Peer {
    pub(crate) metrics: PeerMetrics,
    heartbeat: Heartbeat,
}

impl Peer {
    pub fn new() -> Self {
        Self {
            metrics: PeerMetrics::default(),
            heartbeat: Heartbeat::default(),
        }
    }
}
