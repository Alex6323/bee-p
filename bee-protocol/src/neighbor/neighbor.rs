use crate::message::Heartbeat;
use crate::neighbor::NeighborSenders;
use crate::node::NodeMetrics;

pub(crate) struct Neighbor {
    pub(crate) senders: NeighborSenders,
    pub(crate) metrics: NodeMetrics,
    heartbeat: Heartbeat,
}

impl Neighbor {
    pub fn new(senders: NeighborSenders) -> Self {
        Self {
            senders: senders,
            metrics: NodeMetrics::default(),
            heartbeat: Heartbeat::default(),
        }
    }
}
