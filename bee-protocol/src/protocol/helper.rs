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
    milestone::MilestoneIndex,
    packet::{tlv_into_bytes, Heartbeat, Message as MessagePacket, MessageRequest, MilestoneRequest, Packet},
    protocol::Protocol,
    tangle::MsTangle,
    worker::{MessageRequesterWorkerEvent, MilestoneRequesterWorkerEvent, RequestedMessages, RequestedMilestones},
};

use bee_message::prelude::MessageId;
use bee_network::{Command::SendMessage, EndpointId};
use bee_storage::storage::Backend;

use log::warn;

use std::marker::PhantomData;

pub(crate) struct Sender<P: Packet> {
    marker: PhantomData<P>,
}

macro_rules! implement_sender_worker {
    ($type:ty, $sender:tt, $incrementor:tt) => {
        impl Sender<$type> {
            pub(crate) fn send(epid: &EndpointId, packet: $type) {
                match Protocol::get().network.unbounded_send(SendMessage {
                    receiver_epid: *epid,
                    message: tlv_into_bytes(packet),
                }) {
                    Ok(_) => {
                        // self.peer.metrics.$incrementor();
                        // Protocol::get().metrics.$incrementor();
                    }
                    Err(e) => {
                        warn!("Sending {} to {} failed: {:?}.", stringify!($type), epid, e);
                    }
                }
            }
        }
    };
}

implement_sender_worker!(MilestoneRequest, milestone_request, milestone_requests_sent_inc);
implement_sender_worker!(MessagePacket, message, messages_sent_inc);
implement_sender_worker!(MessageRequest, message_request, message_requests_sent_inc);
implement_sender_worker!(Heartbeat, heartbeat, heartbeats_sent_inc);

impl Protocol {
    // TODO move some functions to workers

    // MilestoneRequest

    pub(crate) fn request_milestone<B: Backend>(
        tangle: &MsTangle<B>,
        milestone_requester: &flume::Sender<MilestoneRequesterWorkerEvent>,
        requested_milestones: &RequestedMilestones,
        index: MilestoneIndex,
        to: Option<EndpointId>,
    ) {
        if !requested_milestones.contains_key(&index) && !tangle.contains_milestone(index) {
            if let Err(e) = milestone_requester.send(MilestoneRequesterWorkerEvent(index, to)) {
                warn!("Requesting milestone failed: {}.", e);
            }
        }
    }

    pub(crate) fn request_latest_milestone<B: Backend>(
        tangle: &MsTangle<B>,
        milestone_requester: &flume::Sender<MilestoneRequesterWorkerEvent>,
        requested_milestones: &RequestedMilestones,
        to: Option<EndpointId>,
    ) {
        Protocol::request_milestone(tangle, milestone_requester, requested_milestones, MilestoneIndex(0), to)
    }

    // MessageRequest

    pub(crate) async fn request_message<B: Backend>(
        tangle: &MsTangle<B>,
        message_requester: &flume::Sender<MessageRequesterWorkerEvent>,
        requested_messages: &RequestedMessages,
        message_id: MessageId,
        index: MilestoneIndex,
    ) {
        if !tangle.contains(&message_id).await
            && !tangle.is_solid_entry_point(&message_id)
            && !requested_messages.contains_key(&message_id)
        {
            if let Err(e) = message_requester.send(MessageRequesterWorkerEvent(message_id, index)) {
                warn!("Requesting message failed: {}.", e);
            }
        }
    }

    // Heartbeat

    pub fn send_heartbeat(
        to: EndpointId,
        latest_solid_milestone_index: MilestoneIndex,
        pruning_milestone_index: MilestoneIndex,
        latest_milestone_index: MilestoneIndex,
    ) {
        Sender::<Heartbeat>::send(
            &to,
            Heartbeat::new(
                *latest_solid_milestone_index,
                *pruning_milestone_index,
                *latest_milestone_index,
                Protocol::get().peer_manager.connected_peers(),
                Protocol::get().peer_manager.synced_peers(),
            ),
        );
    }

    pub fn broadcast_heartbeat(
        latest_solid_milestone_index: MilestoneIndex,
        pruning_milestone_index: MilestoneIndex,
        latest_milestone_index: MilestoneIndex,
    ) {
        for entry in Protocol::get().peer_manager.handshaked_peers.iter() {
            Protocol::send_heartbeat(
                *entry.key(),
                latest_solid_milestone_index,
                pruning_milestone_index,
                latest_milestone_index,
            );
        }
    }
}
