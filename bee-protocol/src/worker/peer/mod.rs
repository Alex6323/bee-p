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

use log::debug;

enum PeerReadState {
    Header,
    Payload(Header),
}

struct PeerReadContext {
    state: PeerReadState,
    buffer: Vec<u8>,
}

struct MessageHandler {
    offset: usize,
    remaining: bool,
    context: PeerReadContext,
    address: Address,
}

impl MessageHandler {
    fn new(address: Address) -> Self {
        Self {
            offset: 0,
            remaining: true,
            context: PeerReadContext {
                state: PeerReadState::Header,
                buffer: vec![],
            },
            address,
        }
    }

    fn get_bytes(&self, begin: usize, end: usize) -> &[u8] {
        &self.context.buffer[begin..end]
    }

    fn append_bytes(&mut self, mut bytes: Vec<u8>) {
        if self.context.buffer.is_empty() {
            self.context.buffer = bytes;
        } else {
            self.context.buffer.append(&mut bytes);
        }
    }

    fn clean_buffer(&mut self) {
        self.context.buffer = self.context.buffer.split_off(self.offset);
    }
}

impl Iterator for MessageHandler {
    type Item = (Header, usize);

    fn next(&mut self) -> Option<Self::Item> {
        while self.remaining {
            match self.context.state {
                PeerReadState::Header => {
                    if self.offset + 3 <= self.context.buffer.len() {
                        debug!("[{}] Reading Header...", self.address);
                        let header = Header::from_bytes(&self.context.buffer[self.offset..self.offset + 3]);
                        self.offset += 3;
                        self.context.state = PeerReadState::Payload(header);
                    } else {
                        self.remaining = false;
                        self.clean_buffer();
                    }
                }
                PeerReadState::Payload(ref header) => {
                    if (self.offset + header.message_length as usize) <= self.context.buffer.len() {
                        let item = Some((header.clone(), self.offset));
                        self.offset += header.message_length as usize;
                        self.context.state = PeerReadState::Header;
                        return item;
                    } else {
                        self.remaining = false;
                        self.clean_buffer();
                    }
                }
            };
        }

        None
    }
}
