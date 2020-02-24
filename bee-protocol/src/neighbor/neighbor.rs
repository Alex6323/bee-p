use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Default)]
pub(crate) struct Neighbor {
    handshake_received: AtomicU32,
    heartbeat_received: AtomicU32,
    legacy_gossip_received: AtomicU32,
    milestone_request_received: AtomicU32,
    transaction_broadcast_received: AtomicU32,
    transaction_request_received: AtomicU32,

    handshake_sent: AtomicU32,
    heartbeat_sent: AtomicU32,
    legacy_gossip_sent: AtomicU32,
    milestone_request_sent: AtomicU32,
    transaction_broadcast_sent: AtomicU32,
    transaction_request_sent: AtomicU32,
}

impl Neighbor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handshake_received(&self) -> u32 {
        self.handshake_received.load(Ordering::Relaxed)
    }

    pub fn handshake_received_inc(&self) -> u32 {
        self.handshake_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn heartbeat_received(&self) -> u32 {
        self.heartbeat_received.load(Ordering::Relaxed)
    }

    pub fn heartbeat_received_inc(&self) -> u32 {
        self.heartbeat_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn legacy_gossip_received(&self) -> u32 {
        self.legacy_gossip_received.load(Ordering::Relaxed)
    }

    pub fn legacy_gossip_received_inc(&self) -> u32 {
        self.legacy_gossip_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn milestone_request_received(&self) -> u32 {
        self.milestone_request_received.load(Ordering::Relaxed)
    }

    pub fn milestone_request_received_inc(&self) -> u32 {
        self.milestone_request_received
            .fetch_add(1, Ordering::SeqCst)
    }

    pub fn transaction_broadcast_received(&self) -> u32 {
        self.transaction_broadcast_received.load(Ordering::Relaxed)
    }

    pub fn transaction_broadcast_received_inc(&self) -> u32 {
        self.transaction_broadcast_received
            .fetch_add(1, Ordering::SeqCst)
    }

    pub fn transaction_request_received(&self) -> u32 {
        self.transaction_request_received.load(Ordering::Relaxed)
    }

    pub fn transaction_request_received_inc(&self) -> u32 {
        self.transaction_request_received
            .fetch_add(1, Ordering::SeqCst)
    }

    pub fn handshake_sent(&self) -> u32 {
        self.handshake_sent.load(Ordering::Relaxed)
    }

    pub fn handshake_sent_inc(&self) -> u32 {
        self.handshake_sent.fetch_add(1, Ordering::SeqCst)
    }

    pub fn heartbeat_sent(&self) -> u32 {
        self.heartbeat_sent.load(Ordering::Relaxed)
    }

    pub fn heartbeat_sent_inc(&self) -> u32 {
        self.heartbeat_sent.fetch_add(1, Ordering::SeqCst)
    }

    pub fn legacy_gossip_sent(&self) -> u32 {
        self.legacy_gossip_sent.load(Ordering::Relaxed)
    }

    pub fn legacy_gossip_sent_inc(&self) -> u32 {
        self.legacy_gossip_sent.fetch_add(1, Ordering::SeqCst)
    }

    pub fn milestone_request_sent(&self) -> u32 {
        self.milestone_request_sent.load(Ordering::Relaxed)
    }

    pub fn milestone_request_sent_inc(&self) -> u32 {
        self.milestone_request_sent.fetch_add(1, Ordering::SeqCst)
    }

    pub fn transaction_broadcast_sent(&self) -> u32 {
        self.transaction_broadcast_sent.load(Ordering::Relaxed)
    }

    pub fn transaction_broadcast_sent_inc(&self) -> u32 {
        self.transaction_broadcast_sent
            .fetch_add(1, Ordering::SeqCst)
    }

    pub fn transaction_request_sent(&self) -> u32 {
        self.transaction_request_sent.load(Ordering::Relaxed)
    }

    pub fn transaction_request_sent_inc(&self) -> u32 {
        self.transaction_request_sent.fetch_add(1, Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn neighbor_counters_received_test() {
        let neighbor = Neighbor::new();

        assert_eq!(neighbor.handshake_received(), 0);
        assert_eq!(neighbor.heartbeat_received(), 0);
        assert_eq!(neighbor.legacy_gossip_received(), 0);
        assert_eq!(neighbor.milestone_request_received(), 0);
        assert_eq!(neighbor.transaction_broadcast_received(), 0);
        assert_eq!(neighbor.transaction_request_received(), 0);

        neighbor.handshake_received_inc();
        neighbor.heartbeat_received_inc();
        neighbor.legacy_gossip_received_inc();
        neighbor.milestone_request_received_inc();
        neighbor.transaction_broadcast_received_inc();
        neighbor.transaction_request_received_inc();

        assert_eq!(neighbor.handshake_received(), 1);
        assert_eq!(neighbor.heartbeat_received(), 1);
        assert_eq!(neighbor.legacy_gossip_received(), 1);
        assert_eq!(neighbor.milestone_request_received(), 1);
        assert_eq!(neighbor.transaction_broadcast_received(), 1);
        assert_eq!(neighbor.transaction_request_received(), 1);
    }

    #[test]
    fn neighbor_counter_sent_test() {
        let neighbor = Neighbor::new();

        assert_eq!(neighbor.handshake_sent(), 0);
        assert_eq!(neighbor.heartbeat_sent(), 0);
        assert_eq!(neighbor.legacy_gossip_sent(), 0);
        assert_eq!(neighbor.milestone_request_sent(), 0);
        assert_eq!(neighbor.transaction_broadcast_sent(), 0);
        assert_eq!(neighbor.transaction_request_sent(), 0);

        neighbor.handshake_sent_inc();
        neighbor.heartbeat_sent_inc();
        neighbor.legacy_gossip_sent_inc();
        neighbor.milestone_request_sent_inc();
        neighbor.transaction_broadcast_sent_inc();
        neighbor.transaction_request_sent_inc();

        assert_eq!(neighbor.handshake_sent(), 1);
        assert_eq!(neighbor.heartbeat_sent(), 1);
        assert_eq!(neighbor.legacy_gossip_sent(), 1);
        assert_eq!(neighbor.milestone_request_sent(), 1);
        assert_eq!(neighbor.transaction_broadcast_sent(), 1);
        assert_eq!(neighbor.transaction_request_sent(), 1);
    }
}
