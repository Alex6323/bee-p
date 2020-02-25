use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Default)]
pub(crate) struct NodeMetrics {
    all_transactions: AtomicU32,
    invalid_transactions: AtomicU32,
    stale_transactions: AtomicU32,
    random_transactions: AtomicU32,
    sent_transactions: AtomicU32,
    new_transactions: AtomicU32,

    handshake_received: AtomicU32,
    legacy_gossip_received: AtomicU32,
    milestone_request_received: AtomicU32,
    transaction_broadcast_received: AtomicU32,
    transaction_request_received: AtomicU32,
    heartbeat_received: AtomicU32,

    handshake_sent: AtomicU32,
    legacy_gossip_sent: AtomicU32,
    milestone_request_sent: AtomicU32,
    transaction_broadcast_sent: AtomicU32,
    transaction_request_sent: AtomicU32,
    heartbeat_sent: AtomicU32,
}

impl NodeMetrics {
    pub fn all_transactions(&self) -> u32 {
        self.all_transactions.load(Ordering::Relaxed)
    }

    pub fn all_transactions_inc(&self) -> u32 {
        self.all_transactions.fetch_add(1, Ordering::SeqCst)
    }

    pub fn invalid_transactions(&self) -> u32 {
        self.invalid_transactions.load(Ordering::Relaxed)
    }

    pub fn invalid_transactions_inc(&self) -> u32 {
        self.invalid_transactions.fetch_add(1, Ordering::SeqCst)
    }

    pub fn stale_transactions(&self) -> u32 {
        self.stale_transactions.load(Ordering::Relaxed)
    }

    pub fn stale_transactions_inc(&self) -> u32 {
        self.stale_transactions.fetch_add(1, Ordering::SeqCst)
    }

    pub fn random_transactions(&self) -> u32 {
        self.random_transactions.load(Ordering::Relaxed)
    }

    pub fn random_transactions_inc(&self) -> u32 {
        self.random_transactions.fetch_add(1, Ordering::SeqCst)
    }

    pub fn sent_transactions(&self) -> u32 {
        self.sent_transactions.load(Ordering::Relaxed)
    }

    pub fn sent_transactions_inc(&self) -> u32 {
        self.sent_transactions.fetch_add(1, Ordering::SeqCst)
    }

    pub fn new_transactions(&self) -> u32 {
        self.new_transactions.load(Ordering::Relaxed)
    }

    pub fn new_transactions_inc(&self) -> u32 {
        self.new_transactions.fetch_add(1, Ordering::SeqCst)
    }

    pub fn handshake_received(&self) -> u32 {
        self.handshake_received.load(Ordering::Relaxed)
    }

    pub fn handshake_received_inc(&self) -> u32 {
        self.handshake_received.fetch_add(1, Ordering::SeqCst)
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

    pub fn heartbeat_received(&self) -> u32 {
        self.heartbeat_received.load(Ordering::Relaxed)
    }

    pub fn heartbeat_received_inc(&self) -> u32 {
        self.heartbeat_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn handshake_sent(&self) -> u32 {
        self.handshake_sent.load(Ordering::Relaxed)
    }

    pub fn handshake_sent_inc(&self) -> u32 {
        self.handshake_sent.fetch_add(1, Ordering::SeqCst)
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

    pub fn heartbeat_sent(&self) -> u32 {
        self.heartbeat_sent.load(Ordering::Relaxed)
    }

    pub fn heartbeat_sent_inc(&self) -> u32 {
        self.heartbeat_sent.fetch_add(1, Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn node_metrics_transactions_test() {
        let metrics = NodeMetrics::default();

        assert_eq!(metrics.all_transactions(), 0);
        assert_eq!(metrics.invalid_transactions(), 0);
        assert_eq!(metrics.stale_transactions(), 0);
        assert_eq!(metrics.random_transactions(), 0);
        assert_eq!(metrics.sent_transactions(), 0);
        assert_eq!(metrics.new_transactions(), 0);

        metrics.all_transactions_inc();
        metrics.invalid_transactions_inc();
        metrics.stale_transactions_inc();
        metrics.random_transactions_inc();
        metrics.sent_transactions_inc();
        metrics.new_transactions_inc();

        assert_eq!(metrics.all_transactions(), 1);
        assert_eq!(metrics.invalid_transactions(), 1);
        assert_eq!(metrics.stale_transactions(), 1);
        assert_eq!(metrics.random_transactions(), 1);
        assert_eq!(metrics.sent_transactions_inc(), 1);
        assert_eq!(metrics.new_transactions(), 1);
    }

    #[test]
    fn node_metrics_received_messages_test() {
        let metrics = NodeMetrics::default();

        assert_eq!(metrics.handshake_received(), 0);
        assert_eq!(metrics.legacy_gossip_received(), 0);
        assert_eq!(metrics.milestone_request_received(), 0);
        assert_eq!(metrics.transaction_broadcast_received(), 0);
        assert_eq!(metrics.transaction_request_received(), 0);
        assert_eq!(metrics.heartbeat_received(), 0);

        metrics.handshake_received_inc();
        metrics.legacy_gossip_received_inc();
        metrics.milestone_request_received_inc();
        metrics.transaction_broadcast_received_inc();
        metrics.transaction_request_received_inc();
        metrics.heartbeat_received_inc();

        assert_eq!(metrics.handshake_received(), 1);
        assert_eq!(metrics.legacy_gossip_received(), 1);
        assert_eq!(metrics.milestone_request_received(), 1);
        assert_eq!(metrics.transaction_broadcast_received(), 1);
        assert_eq!(metrics.transaction_request_received(), 1);
        assert_eq!(metrics.heartbeat_received(), 1);
    }

    #[test]
    fn node_metrics_sent_messages_test() {
        let metrics = NodeMetrics::default();

        assert_eq!(metrics.handshake_sent(), 0);
        assert_eq!(metrics.legacy_gossip_sent(), 0);
        assert_eq!(metrics.milestone_request_sent(), 0);
        assert_eq!(metrics.transaction_broadcast_sent(), 0);
        assert_eq!(metrics.transaction_request_sent(), 0);
        assert_eq!(metrics.heartbeat_sent(), 0);

        metrics.handshake_sent_inc();
        metrics.legacy_gossip_sent_inc();
        metrics.milestone_request_sent_inc();
        metrics.transaction_broadcast_sent_inc();
        metrics.transaction_request_sent_inc();
        metrics.heartbeat_sent_inc();

        assert_eq!(metrics.handshake_sent(), 1);
        assert_eq!(metrics.legacy_gossip_sent(), 1);
        assert_eq!(metrics.milestone_request_sent(), 1);
        assert_eq!(metrics.transaction_broadcast_sent(), 1);
        assert_eq!(metrics.transaction_request_sent(), 1);
        assert_eq!(metrics.heartbeat_sent(), 1);
    }
}
