use crate::message::Heartbeat;
use crate::node::NodeMetrics;

pub(crate) struct Neighbor {
    pub(crate) metrics: NodeMetrics,
    heartbeat: Heartbeat,
}

impl Neighbor {
    pub fn new() -> Self {
        Self {
            metrics: NodeMetrics::default(),
            heartbeat: Heartbeat::default(),
        }
    }
}
