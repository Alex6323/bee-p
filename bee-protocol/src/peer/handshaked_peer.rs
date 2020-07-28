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
    protocol::ProtocolMetrics,
};

use bee_network::{Address, EndpointId};

use std::sync::{
    atomic::{AtomicU32, Ordering},
    Mutex,
};

use futures::channel::{mpsc, oneshot};

pub struct HandshakedPeer {
    pub(crate) epid: EndpointId,
    pub(crate) address: Address,
    pub(crate) metrics: ProtocolMetrics,
    pub(crate) solid_milestone_index: AtomicU32,
    pub(crate) snapshot_milestone_index: AtomicU32,
    pub(crate) milestone_request: (mpsc::Sender<MilestoneRequest>, Mutex<Option<oneshot::Sender<()>>>),
    pub(crate) transaction: (mpsc::Sender<TransactionMessage>, Mutex<Option<oneshot::Sender<()>>>),
    pub(crate) transaction_request: (mpsc::Sender<TransactionRequest>, Mutex<Option<oneshot::Sender<()>>>),
    pub(crate) heartbeat: (mpsc::Sender<Heartbeat>, Mutex<Option<oneshot::Sender<()>>>),
}

impl HandshakedPeer {
    pub(crate) fn new(
        epid: EndpointId,
        address: Address,
        milestone_request: (mpsc::Sender<MilestoneRequest>, Mutex<Option<oneshot::Sender<()>>>),
        transaction: (mpsc::Sender<TransactionMessage>, Mutex<Option<oneshot::Sender<()>>>),
        transaction_request: (mpsc::Sender<TransactionRequest>, Mutex<Option<oneshot::Sender<()>>>),
        heartbeat: (mpsc::Sender<Heartbeat>, Mutex<Option<oneshot::Sender<()>>>),
    ) -> Self {
        Self {
            epid,
            address,
            metrics: ProtocolMetrics::default(),
            solid_milestone_index: AtomicU32::new(0),
            snapshot_milestone_index: AtomicU32::new(0),
            milestone_request,
            transaction,
            transaction_request,
            heartbeat,
        }
    }

    pub(crate) fn set_solid_milestone_index(&self, index: MilestoneIndex) {
        self.solid_milestone_index.store(*index, Ordering::Relaxed);
    }

    pub(crate) fn solid_milestone_index(&self) -> MilestoneIndex {
        self.solid_milestone_index.load(Ordering::Relaxed).into()
    }

    pub(crate) fn set_snapshot_milestone_index(&self, index: MilestoneIndex) {
        self.snapshot_milestone_index.store(*index, Ordering::Relaxed);
    }

    pub(crate) fn snapshot_milestone_index(&self) -> MilestoneIndex {
        self.snapshot_milestone_index.load(Ordering::Relaxed).into()
    }
}
