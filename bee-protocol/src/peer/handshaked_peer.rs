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

use crate::{
    message::{Heartbeat, MilestoneRequest, Transaction as TransactionMessage, TransactionRequest},
    milestone::MilestoneIndex,
    peer::PeerMetrics,
};

use bee_network::{Address, EndpointId};

use std::sync::{
    atomic::{AtomicU32, AtomicU8, Ordering},
    Mutex,
};

use futures::channel::{mpsc, oneshot};

pub struct HandshakedPeer {
    pub(crate) epid: EndpointId,
    pub(crate) address: Address,
    pub(crate) metrics: PeerMetrics,
    pub(crate) latest_solid_milestone_index: AtomicU32,
    pub(crate) snapshot_milestone_index: AtomicU32,
    pub(crate) latest_milestone_index: AtomicU32,
    pub(crate) connected_peers: AtomicU8,
    pub(crate) synced_peers: AtomicU8,
    pub(crate) milestone_request: (
        mpsc::UnboundedSender<MilestoneRequest>,
        Mutex<Option<oneshot::Sender<()>>>,
    ),
    pub(crate) transaction: (
        mpsc::UnboundedSender<TransactionMessage>,
        Mutex<Option<oneshot::Sender<()>>>,
    ),
    pub(crate) transaction_request: (
        mpsc::UnboundedSender<TransactionRequest>,
        Mutex<Option<oneshot::Sender<()>>>,
    ),
    pub(crate) heartbeat: (mpsc::UnboundedSender<Heartbeat>, Mutex<Option<oneshot::Sender<()>>>),
}

impl HandshakedPeer {
    pub(crate) fn new(
        epid: EndpointId,
        address: Address,
        milestone_request: (
            mpsc::UnboundedSender<MilestoneRequest>,
            Mutex<Option<oneshot::Sender<()>>>,
        ),
        transaction: (
            mpsc::UnboundedSender<TransactionMessage>,
            Mutex<Option<oneshot::Sender<()>>>,
        ),
        transaction_request: (
            mpsc::UnboundedSender<TransactionRequest>,
            Mutex<Option<oneshot::Sender<()>>>,
        ),
        heartbeat: (mpsc::UnboundedSender<Heartbeat>, Mutex<Option<oneshot::Sender<()>>>),
    ) -> Self {
        Self {
            epid,
            address,
            metrics: PeerMetrics::default(),
            latest_solid_milestone_index: AtomicU32::new(0),
            snapshot_milestone_index: AtomicU32::new(0),
            latest_milestone_index: AtomicU32::new(0),
            connected_peers: AtomicU8::new(0),
            synced_peers: AtomicU8::new(0),
            milestone_request,
            transaction,
            transaction_request,
            heartbeat,
        }
    }

    pub(crate) fn set_latest_solid_milestone_index(&self, index: MilestoneIndex) {
        self.latest_solid_milestone_index.store(*index, Ordering::Relaxed);
    }

    pub(crate) fn latest_solid_milestone_index(&self) -> MilestoneIndex {
        self.latest_solid_milestone_index.load(Ordering::Relaxed).into()
    }

    pub(crate) fn set_snapshot_milestone_index(&self, index: MilestoneIndex) {
        self.snapshot_milestone_index.store(*index, Ordering::Relaxed);
    }

    pub(crate) fn snapshot_milestone_index(&self) -> MilestoneIndex {
        self.snapshot_milestone_index.load(Ordering::Relaxed).into()
    }

    pub(crate) fn set_latest_milestone_index(&self, index: MilestoneIndex) {
        self.latest_milestone_index.store(*index, Ordering::Relaxed);
    }

    pub(crate) fn latest_milestone_index(&self) -> MilestoneIndex {
        self.latest_milestone_index.load(Ordering::Relaxed).into()
    }

    pub(crate) fn set_connected_peers(&self, connected_peers: u8) {
        self.connected_peers.store(connected_peers, Ordering::Relaxed);
    }

    pub(crate) fn connected_peers(&self) -> u8 {
        self.connected_peers.load(Ordering::Relaxed)
    }

    pub(crate) fn set_synced_peers(&self, synced_peers: u8) {
        self.synced_peers.store(synced_peers, Ordering::Relaxed);
    }

    pub(crate) fn synced_peers(&self) -> u8 {
        self.synced_peers.load(Ordering::Relaxed)
    }

    pub(crate) fn is_solid_at(&self, index: MilestoneIndex) -> bool {
        index > self.snapshot_milestone_index() && index <= self.latest_solid_milestone_index()
    }
}
