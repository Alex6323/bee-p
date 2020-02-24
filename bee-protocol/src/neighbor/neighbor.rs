use crate::neighbor::NeighborMetrics;
use crate::neighbor::NeighborQueues;

#[derive(Default)]
pub(crate) struct Neighbor {
    queues: NeighborQueues,
    metrics: NeighborMetrics,
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
