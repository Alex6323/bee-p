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
    invalid_transactions: AtomicU64,
    new_transactions: AtomicU64,
    known_transactions: AtomicU64,

    invalid_messages: AtomicU64,

    milestone_requests_received: AtomicU64,
    transactions_received: AtomicU64,
    transaction_requests_received: AtomicU64,
    heartbeats_received: AtomicU64,

    milestone_requests_sent: AtomicU64,
    transactions_sent: AtomicU64,
    transaction_requests_sent: AtomicU64,
    heartbeats_sent: AtomicU64,

    value_bundles: AtomicU64,
    non_value_bundles: AtomicU64,
    confirmed_bundles: AtomicU64,
    conflicting_bundles: AtomicU64,
}

impl ProtocolMetrics {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ProtocolMetrics {
    pub fn invalid_transactions(&self) -> u64 {
        self.invalid_transactions.load(Ordering::Relaxed)
    }

    pub(crate) fn invalid_transactions_inc(&self) -> u64 {
        self.invalid_transactions.fetch_add(1, Ordering::SeqCst)
    }

    pub fn new_transactions(&self) -> u64 {
        self.new_transactions.load(Ordering::Relaxed)
    }

    pub(crate) fn new_transactions_inc(&self) -> u64 {
        self.new_transactions.fetch_add(1, Ordering::SeqCst)
    }

    pub fn known_transactions(&self) -> u64 {
        self.known_transactions.load(Ordering::Relaxed)
    }

    pub(crate) fn known_transactions_inc(&self) -> u64 {
        self.known_transactions.fetch_add(1, Ordering::SeqCst)
    }

    pub fn invalid_messages(&self) -> u64 {
        self.invalid_messages.load(Ordering::Relaxed)
    }

    pub(crate) fn invalid_messages_inc(&self) -> u64 {
        self.invalid_messages.fetch_add(1, Ordering::SeqCst)
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub(crate) fn transaction_requests_sent_inc(&self) -> u64 {
        self.transaction_requests_sent.fetch_add(1, Ordering::SeqCst)
    }

    pub fn heartbeats_sent(&self) -> u64 {
        self.heartbeats_sent.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub(crate) fn heartbeats_sent_inc(&self) -> u64 {
        self.heartbeats_sent.fetch_add(1, Ordering::SeqCst)
    }

    pub fn value_bundles(&self) -> u64 {
        self.value_bundles.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub(crate) fn value_bundles_inc(&self) -> u64 {
        self.value_bundles.fetch_add(1, Ordering::SeqCst)
    }

    pub fn non_value_bundles(&self) -> u64 {
        self.non_value_bundles.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub(crate) fn non_value_bundles_inc(&self) -> u64 {
        self.non_value_bundles.fetch_add(1, Ordering::SeqCst)
    }

    pub fn confirmed_bundles(&self) -> u64 {
        self.confirmed_bundles.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub(crate) fn confirmed_bundles_inc(&self) -> u64 {
        self.confirmed_bundles.fetch_add(1, Ordering::SeqCst)
    }

    pub fn conflicting_bundles(&self) -> u64 {
        self.conflicting_bundles.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub(crate) fn conflicting_bundles_inc(&self) -> u64 {
        self.conflicting_bundles.fetch_add(1, Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn protocol_metrics_transactions() {
        let metrics = ProtocolMetrics::default();

        assert_eq!(metrics.invalid_transactions(), 0);
        assert_eq!(metrics.new_transactions(), 0);
        assert_eq!(metrics.known_transactions(), 0);

        metrics.invalid_transactions_inc();
        metrics.new_transactions_inc();
        metrics.known_transactions_inc();

        assert_eq!(metrics.invalid_transactions(), 1);
        assert_eq!(metrics.new_transactions(), 1);
        assert_eq!(metrics.known_transactions(), 1);
    }

    #[test]
    fn protocol_metrics_messages_received() {
        let metrics = ProtocolMetrics::default();

        assert_eq!(metrics.invalid_messages(), 0);
        assert_eq!(metrics.milestone_requests_received(), 0);
        assert_eq!(metrics.transactions_received(), 0);
        assert_eq!(metrics.transaction_requests_received(), 0);
        assert_eq!(metrics.heartbeats_received(), 0);

        metrics.invalid_messages_inc();
        metrics.milestone_requests_received_inc();
        metrics.transactions_received_inc();
        metrics.transaction_requests_received_inc();
        metrics.heartbeats_received_inc();

        assert_eq!(metrics.invalid_messages(), 1);
        assert_eq!(metrics.milestone_requests_received(), 1);
        assert_eq!(metrics.transactions_received(), 1);
        assert_eq!(metrics.transaction_requests_received(), 1);
        assert_eq!(metrics.heartbeats_received(), 1);
    }

    #[test]
    fn protocol_metrics_messages_sent() {
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

    #[test]
    fn protocol_metrics_confirmation() {
        let metrics = ProtocolMetrics::default();

        assert_eq!(metrics.value_bundles(), 0);
        assert_eq!(metrics.non_value_bundles(), 0);
        assert_eq!(metrics.confirmed_bundles(), 0);
        assert_eq!(metrics.conflicting_bundles(), 0);

        metrics.value_bundles_inc();
        metrics.non_value_bundles_inc();
        metrics.confirmed_bundles_inc();
        metrics.conflicting_bundles_inc();

        assert_eq!(metrics.value_bundles(), 1);
        assert_eq!(metrics.non_value_bundles(), 1);
        assert_eq!(metrics.confirmed_bundles(), 1);
        assert_eq!(metrics.conflicting_bundles(), 1);
    }
}
