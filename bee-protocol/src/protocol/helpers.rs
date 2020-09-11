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
    tangle::tangle,
    worker::{BroadcasterWorkerEvent, MilestoneRequesterWorkerEntry, SenderWorker, TransactionRequesterWorkerEntry},
};

use bee_crypto::ternary::Hash;
use bee_network::EndpointId;
use bee_tangle::traversal;

use log::warn;

impl Protocol {
    // MilestoneRequest

    pub fn request_milestone(index: MilestoneIndex, to: Option<EndpointId>) {
        if !Protocol::get().requested_milestones.contains_key(&index) && !tangle().contains_milestone(index) {
            Protocol::get()
                .milestone_requester_worker
                .push(MilestoneRequesterWorkerEntry(index, to));
        }
    }

    pub fn request_latest_milestone(to: Option<EndpointId>) {
        Protocol::request_milestone(MilestoneIndex(0), to);
    }

    pub fn milestone_requester_is_empty() -> bool {
        Protocol::get().milestone_requester_worker.is_empty()
    }

    // TransactionMessage

    pub fn send_transaction(to: EndpointId, transaction: &[u8]) {
        SenderWorker::<TransactionMessage>::send(&to, TransactionMessage::new(transaction));
    }

    // This doesn't use `send_transaction` because answering a request and broadcasting are different priorities
    pub(crate) fn broadcast_transaction_message(source: Option<EndpointId>, transaction: TransactionMessage) {
        if let Err(e) = Protocol::get()
            .broadcaster_worker
            .unbounded_send(BroadcasterWorkerEvent { source, transaction })
        {
            warn!("Broadcasting transaction failed: {}.", e);
        }
    }

    // This doesn't use `send_transaction` because answering a request and broadcasting are different priorities
    pub fn broadcast_transaction(source: Option<EndpointId>, transaction: &[u8]) {
        Protocol::broadcast_transaction_message(source, TransactionMessage::new(transaction));
    }

    // TransactionRequest

    pub fn request_transaction(hash: Hash, index: MilestoneIndex) {
        if !tangle().contains(&hash)
            && !tangle().is_solid_entry_point(&hash)
            && !Protocol::get().requested_transactions.contains_key(&hash)
        {
            Protocol::get()
                .transaction_requester_worker
                .push(TransactionRequesterWorkerEntry(hash, index));
        }
    }

    pub fn transaction_requester_is_empty() -> bool {
        Protocol::get().transaction_requester_worker.is_empty()
    }

    // Heartbeat

    pub fn send_heartbeat(
        to: EndpointId,
        latest_solid_milestone_index: MilestoneIndex,
        snapshot_milestone_index: MilestoneIndex,
        latest_milestone_index: MilestoneIndex,
    ) {
        SenderWorker::<Heartbeat>::send(
            &to,
            Heartbeat::new(
                *latest_solid_milestone_index,
                *snapshot_milestone_index,
                *latest_milestone_index,
                0,
                0,
            ),
        );
    }

    pub fn broadcast_heartbeat(
        latest_solid_milestone_index: MilestoneIndex,
        snapshot_milestone_index: MilestoneIndex,
        latest_milestone_index: MilestoneIndex,
    ) {
        for entry in Protocol::get().peer_manager.handshaked_peers.iter() {
            Protocol::send_heartbeat(
                *entry.key(),
                latest_solid_milestone_index,
                snapshot_milestone_index,
                latest_milestone_index,
            )
        }
    }

    // Solidifier

    pub fn trigger_milestone_solidification(target_index: MilestoneIndex) {
        if let Some(target_hash) = tangle().get_milestone_hash(target_index) {
            if !tangle().is_solid_transaction(&target_hash) {
                traversal::visit_parents_depth_first(
                    tangle(),
                    target_hash,
                    |hash, _, metadata| {
                        !metadata.flags.is_solid() && !Protocol::get().requested_transactions.contains_key(&hash)
                    },
                    |_, _, _| {},
                    |_, _, _| {},
                    |missing_hash| Protocol::request_transaction(*missing_hash, target_index),
                );
            }
        }
    }
}
