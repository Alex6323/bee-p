use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Default)]
pub struct Neighbor {
    handshake_count: AtomicU32,
    heartbeat_count: AtomicU32,
    legacy_gossip_count: AtomicU32,
    milestone_request_count: AtomicU32,
    transaction_broadcast_count: AtomicU32,
    transaction_request_count: AtomicU32,
}

impl Neighbor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handshake_count(&self) -> u32 {
        self.handshake_count.load(Ordering::Relaxed)
    }

    pub fn handshake_count_inc(&self) -> u32 {
        self.handshake_count.fetch_add(1, Ordering::SeqCst)
    }

    pub fn heartbeat_count(&self) -> u32 {
        self.heartbeat_count.load(Ordering::Relaxed)
    }

    pub fn heartbeat_count_inc(&self) -> u32 {
        self.heartbeat_count.fetch_add(1, Ordering::SeqCst)
    }

    pub fn legacy_gossip_count(&self) -> u32 {
        self.legacy_gossip_count.load(Ordering::Relaxed)
    }

    pub fn legacy_gossip_count_inc(&self) -> u32 {
        self.legacy_gossip_count.fetch_add(1, Ordering::SeqCst)
    }

    pub fn milestone_request_count(&self) -> u32 {
        self.milestone_request_count.load(Ordering::Relaxed)
    }

    pub fn milestone_request_count_inc(&self) -> u32 {
        self.milestone_request_count.fetch_add(1, Ordering::SeqCst)
    }

    pub fn transaction_broadcast_count(&self) -> u32 {
        self.transaction_broadcast_count.load(Ordering::Relaxed)
    }

    pub fn transaction_broadcast_count_inc(&self) -> u32 {
        self.transaction_broadcast_count
            .fetch_add(1, Ordering::SeqCst)
    }

    pub fn transaction_request_count(&self) -> u32 {
        self.transaction_request_count.load(Ordering::Relaxed)
    }

    pub fn transaction_request_count_inc(&self) -> u32 {
        self.transaction_request_count
            .fetch_add(1, Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn neighbor_counter_test() {
        let neighbor = Neighbor::new();

        assert_eq!(neighbor.handshake_count(), 0);
        assert_eq!(neighbor.heartbeat_count(), 0);
        assert_eq!(neighbor.legacy_gossip_count(), 0);
        assert_eq!(neighbor.milestone_request_count(), 0);
        assert_eq!(neighbor.transaction_broadcast_count(), 0);
        assert_eq!(neighbor.transaction_request_count(), 0);

        neighbor.handshake_count_inc();
        neighbor.heartbeat_count_inc();
        neighbor.legacy_gossip_count_inc();
        neighbor.milestone_request_count_inc();
        neighbor.transaction_broadcast_count_inc();
        neighbor.transaction_request_count_inc();

        assert_eq!(neighbor.handshake_count(), 1);
        assert_eq!(neighbor.heartbeat_count(), 1);
        assert_eq!(neighbor.legacy_gossip_count(), 1);
        assert_eq!(neighbor.milestone_request_count(), 1);
        assert_eq!(neighbor.transaction_broadcast_count(), 1);
        assert_eq!(neighbor.transaction_request_count(), 1);
    }
}
