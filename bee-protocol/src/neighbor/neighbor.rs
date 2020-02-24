use crate::neighbor::NeighborMetrics;

#[derive(Default)]
pub(crate) struct Neighbor {
    pub(crate) metrics: NeighborMetrics,
}

impl Neighbor {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
}
