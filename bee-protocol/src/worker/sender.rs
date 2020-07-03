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
    message::{
        tlv_into_bytes, Heartbeat, Message, MilestoneRequest, Transaction as TransactionMessage, TransactionRequest,
    },
    peer::HandshakedPeer,
    protocol::Protocol,
};

use bee_network::{Command::SendMessage, EndpointId, Network};

use std::{marker::PhantomData, sync::Arc};

use futures::{
    channel::{mpsc, oneshot},
    future::FutureExt,
    select,
    sink::SinkExt,
    stream::StreamExt,
};
use log::warn;

pub(crate) struct SenderWorker<M: Message> {
    network: Network,
    peer: Arc<HandshakedPeer>,
    _message_type: PhantomData<M>,
}

macro_rules! implement_sender_worker {
    ($type:ty, $sender:tt, $incrementor:tt) => {
        impl SenderWorker<$type> {
            pub(crate) fn new(network: Network, peer: Arc<HandshakedPeer>) -> Self {
                Self {
                    network,
                    peer,
                    _message_type: PhantomData,
                }
            }

            pub(crate) async fn send(epid: &EndpointId, message: $type) {
                if let Some(context) = Protocol::get().peer_manager.handshaked_peers.get(&epid) {
                    if let Err(e) = context
                        .$sender
                        .0
                        // TODO avoid clone ?
                        .clone()
                        .send(message)
                        .await
                    {
                        // TODO log actual message type ?
                        warn!("Sending message to {} failed: {:?}.", epid, e);
                    }
                };
            }

            pub(crate) async fn run(
                mut self,
                events_receiver: mpsc::Receiver<$type>,
                shutdown_receiver: oneshot::Receiver<()>,
            ) {
                let mut events_fused = events_receiver.fuse();
                let mut shutdown_fused = shutdown_receiver.fuse();

                loop {
                    select! {
                        message = events_fused.next() => {
                            if let Some(message) = message {
                                match self
                                    .network
                                    .send(SendMessage {
                                        epid: self.peer.epid,
                                        bytes: tlv_into_bytes(message),
                                        responder: None,
                                    })
                                    .await
                                {
                                    Ok(_) => {
                                        self.peer.metrics.$incrementor();
                                        Protocol::get().metrics.$incrementor();
                                    }
                                    Err(e) => {
                                        // TODO log actual message type ?
                                        warn!(
                                            "Sending message to {} failed: {:?}.",
                                            self.peer.epid, e
                                        );
                                    }
                                }
                            }
                        }
                        _ = shutdown_fused => {
                            break;
                        }
                    }
                }
            }
        }
    };
}

implement_sender_worker!(MilestoneRequest, milestone_request, milestone_request_sent_inc);
implement_sender_worker!(TransactionMessage, transaction, transaction_sent_inc);
implement_sender_worker!(TransactionRequest, transaction_request, transaction_request_sent_inc);
implement_sender_worker!(Heartbeat, heartbeat, heartbeat_sent_inc);

// TODO is this really necessary ?
