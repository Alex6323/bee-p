use crate::neighbor::NeighborQueues;
use crate::node::NodeMetrics;

#[derive(Default)]
pub(crate) struct Neighbor {
    queues: NeighborQueues,
    metrics: NodeMetrics,
}

impl Neighbor {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn neighbor_test() {
        let neighbor = Neighbor::new();
    }
}
