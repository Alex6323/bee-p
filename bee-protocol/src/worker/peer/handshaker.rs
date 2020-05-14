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
    message::{tlv_from_bytes, tlv_into_bytes, Handshake, Header, Message, MESSAGES_VERSIONS},
    peer::Peer,
    protocol::Protocol,
    worker::{peer::validate_handshake, PeerWorker},
};

use bee_network::{Command::SendMessage, Network};
use bee_tangle::tangle;

use std::sync::Arc;

use async_std::task::spawn;
use futures::{
    channel::{mpsc, oneshot},
    future::FutureExt,
    select,
    stream::StreamExt,
};
use log::{debug, error, info, warn};

#[derive(Debug)]
pub(crate) enum PeerHandshakerWorkerError {}

enum PeerReadState {
    Header,
    Payload(Header),
}

struct PeerReadContext {
    state: PeerReadState,
    buffer: Vec<u8>,
}

pub struct PeerHandshakerWorker {
    network: Network,
    peer: Arc<Peer>,
    handshaked: bool,
}

impl PeerHandshakerWorker {
    pub(crate) fn new(network: Network, peer: Arc<Peer>) -> Self {
        Self {
            network,
            peer,
            handshaked: false,
        }
    }

    pub async fn run(mut self, receiver: mpsc::Receiver<Vec<u8>>, shutdown: oneshot::Receiver<()>) {
        info!("[PeerHandshakerWorker({})] Running.", self.peer.epid);

        let mut context = PeerReadContext {
            state: PeerReadState::Header,
            buffer: Vec::new(),
        };
        let mut receiver_fused = receiver.fuse();
        let mut shutdown_fused = shutdown.fuse();

        // This is the only message not using a SenderWorker because they are not running yet (awaiting handshake)
        if let Err(e) = self
            .network
            .send(SendMessage {
                epid: self.peer.epid,
                // TODO port
                bytes: tlv_into_bytes(Handshake::new(
                    1337,
                    &Protocol::get().config.coordinator.public_key_bytes,
                    Protocol::get().config.mwm,
                    &MESSAGES_VERSIONS,
                )),
                responder: None,
            })
            .await
        {
            // TODO then what ?
            warn!(
                "[PeerHandshakerWorker({})] Failed to send handshake: {:?}.",
                self.peer.epid, e
            );
        }

        loop {
            select! {
                event = receiver_fused.next() => {
                    if let Some(event) = event {
                        context = self.message_handler(context, event).await;
                        if self.handshaked {
                            break;
                        }
                    }
                },
                _ = shutdown_fused => {
                    break;
                }
            }
        }

        info!("[PeerHandshakerWorker({})] Stopped.", self.peer.epid);

        spawn(
            PeerWorker::new(
                Protocol::get()
                    .peer_manager
                    .handshaked_peers
                    .get(&self.peer.epid)
                    .unwrap()
                    .value()
                    .clone(),
            )
            .run(receiver_fused, shutdown_fused),
        );
    }

    async fn process_message(&mut self, header: &Header, bytes: &[u8]) -> Result<(), PeerHandshakerWorkerError> {
        if let Handshake::ID = header.message_type {
            debug!("[PeerHandshakerWorker({})] Reading Handshake...", self.peer.epid);
            match tlv_from_bytes::<Handshake>(&header, bytes) {
                Ok(handshake) => match validate_handshake(&self.peer, handshake) {
                    Ok(_) => {
                        info!("[PeerHandshakerWorker({})] Handshake completed.", self.peer.epid);

                        Protocol::get().peer_manager.handshake(&self.peer.epid);

                        Protocol::send_heartbeat(
                            self.peer.epid,
                            *tangle().get_solid_milestone_index(),
                            *tangle().get_snapshot_milestone_index(),
                        )
                        .await;

                        Protocol::request_last_milestone(Some(self.peer.epid));
                        Protocol::trigger_milestone_solidification().await;

                        self.handshaked = true;
                    }
                    Err(_) => {
                        // TODO handle
                    }
                },
                Err(e) => {
                    warn!(
                        "[PeerHandshakerWorker({})] Reading Handshake failed: {:?}.",
                        self.peer.epid, e
                    );

                    Protocol::get().metrics.invalid_messages_received_inc();
                }
            }
        } else {
            warn!(
                "[PeerHandshakerWorker({})] Ignoring messages until fully handshaked.",
                self.peer.epid
            );

            Protocol::get().metrics.invalid_messages_received_inc();
        }

        Ok(())
    }

    async fn message_handler(&mut self, mut context: PeerReadContext, mut bytes: Vec<u8>) -> PeerReadContext {
        let mut offset = 0;
        let mut remaining = true;

        if context.buffer.is_empty() {
            context.buffer = bytes;
        } else {
            context.buffer.append(&mut bytes);
        }

        while remaining {
            context.state = match context.state {
                PeerReadState::Header => {
                    if offset + 3 <= context.buffer.len() {
                        debug!("[PeerHandshakerWorker({})] Reading Header...", self.peer.epid);
                        let header = Header::from_bytes(&context.buffer[offset..offset + 3]);
                        offset = offset + 3;

                        PeerReadState::Payload(header)
                    } else {
                        remaining = false;

                        PeerReadState::Header
                    }
                }
                PeerReadState::Payload(header) => {
                    if (offset + header.message_length as usize) <= context.buffer.len() {
                        if let Err(e) = self
                            .process_message(
                                &header,
                                &context.buffer[offset..offset + header.message_length as usize],
                            )
                            .await
                        {
                            error!(
                                "[PeerHandshakerWorker({})] Processing message failed: {:?}.",
                                self.peer.epid, e
                            );
                        }

                        offset = offset + header.message_length as usize;

                        PeerReadState::Header
                    } else {
                        remaining = false;

                        PeerReadState::Payload(header)
                    }
                }
            };
        }

        PeerReadContext {
            state: context.state,
            buffer: context.buffer[offset..].to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {}
