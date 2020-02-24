use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Default)]
pub(crate) struct NeighborMetrics {
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

impl NeighborMetrics {
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
    fn neighbor_metrics_received_messages_test() {
        let metrics = NeighborMetrics::default();

        assert_eq!(metrics.handshake_received(), 0);
        assert_eq!(metrics.heartbeat_received(), 0);
        assert_eq!(metrics.legacy_gossip_received(), 0);
        assert_eq!(metrics.milestone_request_received(), 0);
        assert_eq!(metrics.transaction_broadcast_received(), 0);
        assert_eq!(metrics.transaction_request_received(), 0);

        metrics.handshake_received_inc();
        metrics.heartbeat_received_inc();
        metrics.legacy_gossip_received_inc();
        metrics.milestone_request_received_inc();
        metrics.transaction_broadcast_received_inc();
        metrics.transaction_request_received_inc();

        assert_eq!(metrics.handshake_received(), 1);
        assert_eq!(metrics.heartbeat_received(), 1);
        assert_eq!(metrics.legacy_gossip_received(), 1);
        assert_eq!(metrics.milestone_request_received(), 1);
        assert_eq!(metrics.transaction_broadcast_received(), 1);
        assert_eq!(metrics.transaction_request_received(), 1);
    }

    #[test]
    fn neighbor_metrics_sent_messages_test() {
        let metrics = NeighborMetrics::default();

        assert_eq!(metrics.handshake_sent(), 0);
        assert_eq!(metrics.heartbeat_sent(), 0);
        assert_eq!(metrics.legacy_gossip_sent(), 0);
        assert_eq!(metrics.milestone_request_sent(), 0);
        assert_eq!(metrics.transaction_broadcast_sent(), 0);
        assert_eq!(metrics.transaction_request_sent(), 0);

        metrics.handshake_sent_inc();
        metrics.heartbeat_sent_inc();
        metrics.legacy_gossip_sent_inc();
        metrics.milestone_request_sent_inc();
        metrics.transaction_broadcast_sent_inc();
        metrics.transaction_request_sent_inc();

        assert_eq!(metrics.handshake_sent(), 1);
        assert_eq!(metrics.heartbeat_sent(), 1);
        assert_eq!(metrics.legacy_gossip_sent(), 1);
        assert_eq!(metrics.milestone_request_sent(), 1);
        assert_eq!(metrics.transaction_broadcast_sent(), 1);
        assert_eq!(metrics.transaction_request_sent(), 1);
    }
}
