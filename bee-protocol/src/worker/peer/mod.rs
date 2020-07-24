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

mod handshaker;
mod peer;

pub(crate) use handshaker::PeerHandshakerWorker;
pub(crate) use peer::PeerWorker;

use crate::message::Header;

use bee_network::Address;

use futures::stream::StreamExt;
use futures::{
    channel::{mpsc, oneshot},
    future, select, stream,
};
use log::debug;

enum PeerReadState {
    Header,
    Payload(Header),
}

struct MessageHandler {
    events: EventHandler,
    state: PeerReadState,
    address: Address,
}

impl MessageHandler {
    fn new(
        receiver_fused: stream::Fuse<mpsc::Receiver<Vec<u8>>>,
        shutdown_fused: future::Fuse<oneshot::Receiver<()>>,
        address: Address,
    ) -> Self {
        Self {
            events: EventHandler::new(receiver_fused, shutdown_fused),
            state: PeerReadState::Header,
            address,
        }
    }

    async fn fetch_message<'a>(&'a mut self) -> Option<(Header, &'a [u8])> {
        loop {
            match &self.state {
                PeerReadState::Header => {
                    let bytes = self.events.fetch_bytes(3).await?;
                    debug!("[{}] Reading Header...", self.address);
                    let header = Header::from_bytes(bytes);
                    self.state = PeerReadState::Payload(header);
                }
                PeerReadState::Payload(header) => {
                    let header = header.clone();
                    let bytes = self.events.fetch_bytes(header.message_length as usize).await?;
                    self.state = PeerReadState::Header;
                    return Some((header, bytes));
                }
            }
        }
    }
}

struct EventHandler {
    receiver_fused: stream::Fuse<mpsc::Receiver<Vec<u8>>>,
    shutdown_fused: future::Fuse<oneshot::Receiver<()>>,
    buffer: Vec<u8>,
    offset: usize,
    closed: bool,
}

impl EventHandler {
    fn new(
        receiver_fused: stream::Fuse<mpsc::Receiver<Vec<u8>>>,
        shutdown_fused: future::Fuse<oneshot::Receiver<()>>,
    ) -> Self {
        Self {
            receiver_fused,
            shutdown_fused,
            buffer: vec![],
            offset: 0,
            closed: false,
        }
    }

    fn push_event(&mut self, mut bytes: Vec<u8>) {
        self.buffer = self.buffer.split_off(self.offset);
        self.offset = 0;

        if self.buffer.is_empty() {
            self.buffer = bytes;
        } else {
            self.buffer.append(&mut bytes);
        }
    }

    async fn fetch_bytes<'a>(&'a mut self, len: usize) -> Option<&'a [u8]> {
        if self.closed {
            return None;
        }

        while self.offset + len > self.buffer.len() {
            select! {
                event = self.receiver_fused.next() => {
                    if let Some(event) = event {
                        self.push_event(event);
                    }
                },
                _ = &mut self.shutdown_fused => {
                    self.closed = true;
                    return None;
                }
            }
        }

        let item = &self.buffer[self.offset..(self.offset + len)];
        self.offset += len;
        Some(item)
    }
}
