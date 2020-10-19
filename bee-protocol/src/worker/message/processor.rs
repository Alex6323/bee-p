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
    config::ProtocolConfig,
    packet::Message as MessagePacket,
    protocol::Protocol,
    tangle::{MessageMetadata, MsTangle},
    worker::{
        BroadcasterWorker, BroadcasterWorkerEvent, MessageRequesterWorker, MilestoneValidatorWorker,
        MilestoneValidatorWorkerEvent, PropagatorWorker, PropagatorWorkerEvent, TangleWorker,
    },
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{node::Node, packable::Packable, worker::Worker};
use bee_message::prelude::{Message, MessageId, Payload};
use bee_network::EndpointId;

use async_trait::async_trait;
use blake2::{Blake2b, Digest};
use futures::stream::StreamExt;
use log::{error, info, trace, warn};

use std::any::TypeId;

pub(crate) struct ProcessorWorkerEvent {
    pub(crate) pow_score: f64,
    pub(crate) from: EndpointId,
    pub(crate) message_packet: MessagePacket,
}

pub(crate) struct ProcessorWorker {
    pub(crate) tx: flume::Sender<ProcessorWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for ProcessorWorker {
    type Config = ProtocolConfig;
    type Error = WorkerError;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<TangleWorker>(),
            TypeId::of::<MilestoneValidatorWorker>(),
            TypeId::of::<PropagatorWorker>(),
            TypeId::of::<BroadcasterWorker>(),
            TypeId::of::<MessageRequesterWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();
        let milestone_validator = node.worker::<MilestoneValidatorWorker>().unwrap().tx.clone();
        let propagator = node.worker::<PropagatorWorker>().unwrap().tx.clone();
        let broadcaster = node.worker::<BroadcasterWorker>().unwrap().tx.clone();
        let message_requester = node.worker::<MessageRequesterWorker>().unwrap().tx.clone();

        let tangle = node.resource::<MsTangle<N::Backend>>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());
            let mut blake2b = Blake2b::default();

            while let Some(ProcessorWorkerEvent {
                pow_score: _pow_score,
                from,
                message_packet,
            }) = receiver.next().await
            {
                trace!("Processing received message...");

                let message = match Message::unpack(&mut &message_packet.bytes[..]) {
                    Ok(message) => {
                        // TODO validation
                        message
                    }
                    Err(e) => {
                        trace!("Invalid message: {:?}.", e);
                        Protocol::get().metrics.invalid_messages_inc();
                        return;
                    }
                };

                blake2b.update(&message_packet.bytes);
                // TODO Do we have to copy ?
                let mut bytes = [0u8; 32];
                bytes.copy_from_slice(&blake2b.finalize_reset());
                let message_id = MessageId::from(bytes);

                let requested = Protocol::get().requested_messages.contains_key(&message_id);

                // TODO when PoW
                // if !requested && message_id.weight() < config.mwm {
                //     trace!("Insufficient weight magnitude: {}.", message_id.weight());
                //     Protocol::get().metrics.invalid_messages_inc();
                //     return;
                // }

                let mut metadata = MessageMetadata::arrived();

                metadata.flags_mut().set_requested(requested);

                // store message
                if let Some(message) = tangle.insert(message, message_id, metadata).await {
                    // TODO this was temporarily moved from the tangle.
                    // Reason is that since the tangle is not a worker, it can't have access to the propagator tx.
                    // When the tangle is made a worker, this should be put back on.

                    if let Err(e) = propagator.send(PropagatorWorkerEvent(message_id)) {
                        error!("Failed to send message id {} to propagator: {:?}.", message_id, e);
                    }

                    Protocol::get().metrics.new_messages_inc();

                    match Protocol::get().requested_messages.remove(&message_id) {
                        Some((_, (index, _))) => {
                            // Message was requested.
                            let parent1 = message.parent1();
                            let parent2 = message.parent2();

                            Protocol::request_message(&tangle, &message_requester, *parent1, index).await;
                            if parent1 != parent2 {
                                Protocol::request_message(&tangle, &message_requester, *parent2, index).await;
                            }
                        }
                        None => {
                            // Message was not requested.
                            if let Err(e) = broadcaster.send(BroadcasterWorkerEvent {
                                source: Some(from),
                                message: message_packet,
                            }) {
                                warn!("Broadcasting message failed: {}.", e);
                            }
                        }
                    };

                    if let Payload::Milestone(_) = message.payload() {
                        if let Err(e) = milestone_validator.send(MilestoneValidatorWorkerEvent(message_id)) {
                            error!(
                                "Sending message id {} to milestone validation failed: {:?}.",
                                message_id, e
                            );
                        }
                    }
                } else {
                    Protocol::get().metrics.known_messages_inc();
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
