// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Default)]
pub struct ProtocolMetrics {
    invalid_transactions_received: AtomicU64,
    stale_transactions_received: AtomicU64,
    new_transactions_received: AtomicU64,
    known_transactions_received: AtomicU64,

    invalid_messages_received: AtomicU64,

    milestone_requests_received: AtomicU64,
    transactions_received: AtomicU64,
    transaction_requests_received: AtomicU64,
    heartbeats_received: AtomicU64,

    milestone_requests_sent: AtomicU64,
    transactions_sent: AtomicU64,
    transaction_requests_sent: AtomicU64,
    heartbeats_sent: AtomicU64,
}

impl ProtocolMetrics {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ProtocolMetrics {
    pub fn invalid_transactions_received(&self) -> u64 {
        self.invalid_transactions_received.load(Ordering::Relaxed)
    }

    pub(crate) fn invalid_transactions_received_inc(&self) -> u64 {
        self.invalid_transactions_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn stale_transactions_received(&self) -> u64 {
        self.stale_transactions_received.load(Ordering::Relaxed)
    }

    pub(crate) fn stale_transactions_received_inc(&self) -> u64 {
        self.stale_transactions_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn new_transactions_received(&self) -> u64 {
        self.new_transactions_received.load(Ordering::Relaxed)
    }

    pub(crate) fn new_transactions_received_inc(&self) -> u64 {
        self.new_transactions_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn known_transactions_received(&self) -> u64 {
        self.known_transactions_received.load(Ordering::Relaxed)
    }

    pub(crate) fn known_transactions_received_inc(&self) -> u64 {
        self.known_transactions_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn invalid_messages_received(&self) -> u64 {
        self.invalid_messages_received.load(Ordering::Relaxed)
    }

    pub(crate) fn invalid_messages_received_inc(&self) -> u64 {
        self.invalid_messages_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn milestone_requests_received(&self) -> u64 {
        self.milestone_requests_received.load(Ordering::Relaxed)
    }

    pub(crate) fn milestone_requests_received_inc(&self) -> u64 {
        self.milestone_requests_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn transactions_received(&self) -> u64 {
        self.transactions_received.load(Ordering::Relaxed)
    }

    pub(crate) fn transactions_received_inc(&self) -> u64 {
        self.transactions_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn transaction_requests_received(&self) -> u64 {
        self.transaction_requests_received.load(Ordering::Relaxed)
    }

    pub(crate) fn transaction_requests_received_inc(&self) -> u64 {
        self.transaction_requests_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn heartbeats_received(&self) -> u64 {
        self.heartbeats_received.load(Ordering::Relaxed)
    }

    pub(crate) fn heartbeats_received_inc(&self) -> u64 {
        self.heartbeats_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn milestone_requests_sent(&self) -> u64 {
        self.milestone_requests_sent.load(Ordering::Relaxed)
    }

    pub(crate) fn milestone_requests_sent_inc(&self) -> u64 {
        self.milestone_requests_sent.fetch_add(1, Ordering::SeqCst)
    }

    pub fn transactions_sent(&self) -> u64 {
        self.transactions_sent.load(Ordering::Relaxed)
    }

    pub(crate) fn transactions_sent_inc(&self) -> u64 {
        self.transactions_sent.fetch_add(1, Ordering::SeqCst)
    }

    pub fn transaction_requests_sent(&self) -> u64 {
        self.transaction_requests_sent.load(Ordering::Relaxed)
    }

    pub(crate) fn transaction_requests_sent_inc(&self) -> u64 {
        self.transaction_requests_sent.fetch_add(1, Ordering::SeqCst)
    }

    pub fn heartbeats_sent(&self) -> u64 {
        self.heartbeats_sent.load(Ordering::Relaxed)
    }

    pub(crate) fn heartbeats_sent_inc(&self) -> u64 {
        self.heartbeats_sent.fetch_add(1, Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn protocol_metrics_transactions_received_test() {
        let metrics = ProtocolMetrics::default();

        assert_eq!(metrics.invalid_transactions_received(), 0);
        assert_eq!(metrics.stale_transactions_received(), 0);
        assert_eq!(metrics.new_transactions_received(), 0);
        assert_eq!(metrics.known_transactions_received(), 0);

        metrics.invalid_transactions_received_inc();
        metrics.stale_transactions_received_inc();
        metrics.new_transactions_received_inc();
        metrics.known_transactions_received_inc();

        assert_eq!(metrics.invalid_transactions_received(), 1);
        assert_eq!(metrics.stale_transactions_received(), 1);
        assert_eq!(metrics.new_transactions_received(), 1);
        assert_eq!(metrics.known_transactions_received(), 1);
    }

    #[test]
    fn protocol_metrics_messages_received_test() {
        let metrics = ProtocolMetrics::default();

        assert_eq!(metrics.invalid_messages_received(), 0);
        assert_eq!(metrics.milestone_requests_received(), 0);
        assert_eq!(metrics.transactions_received(), 0);
        assert_eq!(metrics.transaction_requests_received(), 0);
        assert_eq!(metrics.heartbeats_received(), 0);

        metrics.invalid_messages_received_inc();
        metrics.milestone_requests_received_inc();
        metrics.transactions_received_inc();
        metrics.transaction_requests_received_inc();
        metrics.heartbeats_received_inc();

        assert_eq!(metrics.invalid_messages_received(), 1);
        assert_eq!(metrics.milestone_requests_received(), 1);
        assert_eq!(metrics.transactions_received(), 1);
        assert_eq!(metrics.transaction_requests_received(), 1);
        assert_eq!(metrics.heartbeats_received(), 1);
    }

    #[test]
    fn protocol_metrics_messages_sent_test() {
        let metrics = ProtocolMetrics::default();

        assert_eq!(metrics.milestone_requests_sent(), 0);
        assert_eq!(metrics.transactions_sent(), 0);
        assert_eq!(metrics.transaction_requests_sent(), 0);
        assert_eq!(metrics.heartbeats_sent(), 0);

        metrics.milestone_requests_sent_inc();
        metrics.transactions_sent_inc();
        metrics.transaction_requests_sent_inc();
        metrics.heartbeats_sent_inc();

        assert_eq!(metrics.milestone_requests_sent(), 1);
        assert_eq!(metrics.transactions_sent(), 1);
        assert_eq!(metrics.transaction_requests_sent(), 1);
        assert_eq!(metrics.heartbeats_sent(), 1);
    }
}
