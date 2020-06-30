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
    message::{Heartbeat, Transaction as TransactionMessage},
    milestone::MilestoneIndex,
    protocol::Protocol,
    worker::{
        BroadcasterWorkerEvent, MilestoneRequesterWorkerEntry, MilestoneSolidifierWorkerEvent, SenderWorker,
        TransactionRequesterWorkerEntry, TransactionSolidifierWorkerEvent,
    },
};

use bee_crypto::ternary::Hash;
use bee_network::EndpointId;

use futures::sink::SinkExt;
use log::warn;

impl Protocol {
    // MilestoneRequest

    pub fn request_milestone(index: MilestoneIndex, to: Option<EndpointId>) {
        Protocol::get()
            .milestone_requester_worker
            .0
            .push(MilestoneRequesterWorkerEntry(index, to));
    }

    pub fn request_last_milestone(to: Option<EndpointId>) {
        Protocol::request_milestone(MilestoneIndex(0), to);
    }

    pub fn milestone_requester_is_empty() -> bool {
        Protocol::get().milestone_requester_worker.0.is_empty()
    }

    // TransactionMessage

    pub async fn send_transaction(to: EndpointId, transaction: &[u8]) {
        SenderWorker::<TransactionMessage>::send(&to, TransactionMessage::new(transaction)).await;
    }

    // This doesn't use `send_transaction` because answering a request and broadcasting are different priorities
    pub(crate) async fn broadcast_transaction_message(source: Option<EndpointId>, transaction: TransactionMessage) {
        if let Err(e) = Protocol::get()
            .broadcaster_worker
            .0
            // TODO try to avoid
            .clone()
            .send(BroadcasterWorkerEvent { source, transaction })
            .await
        {
            warn!("Broadcasting transaction failed: {}.", e);
        }
    }

    // This doesn't use `send_transaction` because answering a request and broadcasting are different priorities
    pub async fn broadcast_transaction(source: Option<EndpointId>, transaction: &[u8]) {
        Protocol::broadcast_transaction_message(source, TransactionMessage::new(transaction)).await;
    }

    // TransactionRequest

    pub async fn request_transaction(hash: Hash, index: MilestoneIndex) {
        Protocol::get()
            .transaction_requester_worker
            .0
            .push(TransactionRequesterWorkerEntry(hash, index));
    }

    pub fn transaction_requester_is_empty() -> bool {
        Protocol::get().transaction_requester_worker.0.is_empty()
    }

    // Heartbeat

    pub async fn send_heartbeat(
        to: EndpointId,
        solid_milestone_index: MilestoneIndex,
        snapshot_milestone_index: MilestoneIndex,
    ) {
        SenderWorker::<Heartbeat>::send(&to, Heartbeat::new(*solid_milestone_index, *snapshot_milestone_index)).await;
    }

    pub async fn broadcast_heartbeat(solid_milestone_index: MilestoneIndex, snapshot_milestone_index: MilestoneIndex) {
        for entry in Protocol::get().peer_manager.handshaked_peers.iter() {
            Protocol::send_heartbeat(*entry.key(), solid_milestone_index, snapshot_milestone_index).await;
        }
    }

    // Solidifier

    pub async fn trigger_transaction_solidification(hash: Hash, index: MilestoneIndex) {
        if let Err(e) = Protocol::get()
            .transaction_solidifier_worker
            // TODO try to avoid clone
            .0
            .clone()
            .send(TransactionSolidifierWorkerEvent(hash, index))
            .await
        {
            warn!("Triggering transaction solidification failed: {}.", e);
        }
    }

    pub async fn trigger_milestone_solidification() {
        if let Err(e) = Protocol::get()
            .milestone_solidifier_worker
            // TODO try to avoid clone
            .0
            .clone()
            .send(MilestoneSolidifierWorkerEvent())
            .await
        {
            warn!("Triggering milestone solidification failed: {}.", e);
        }
    }
}
