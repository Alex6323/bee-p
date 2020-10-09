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
    atomic::{
        packable::{Buf, BufMut, Packable},
        payload::Payload,
        Error, MessageId,
    },
    Vertex,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
    parent1: MessageId,
    parent2: MessageId,
    payload: Payload,
    nonce: u64,
}

impl Message {
    pub fn builder() -> MessageBuilder {
        MessageBuilder::new()
    }

    pub fn parent1(&self) -> &MessageId {
        &self.parent1
    }

    pub fn parent2(&self) -> &MessageId {
        &self.parent2
    }

    pub fn payload(&self) -> &Payload {
        &self.payload
    }

    pub fn nonce(&self) -> u64 {
        self.nonce
    }
}

impl Packable for Message {
    fn len_bytes(&self) -> usize {
        0u8.len_bytes()
            + self.parent1.len_bytes()
            + self.parent2.len_bytes()
            + 0u32.len_bytes()
            + self.payload.len_bytes()
            + 0u64.len_bytes()
    }

    fn pack<B: BufMut>(&self, buffer: &mut B) {
        1u8.pack(buffer);

        self.parent1.pack(buffer);
        self.parent2.pack(buffer);

        (self.payload.len_bytes() as u32).pack(buffer);
        self.payload.pack(buffer);

        self.nonce.pack(buffer);
    }

    fn unpack<B: Buf>(buffer: &mut B) -> Self {
        assert_eq!(1u8, u8::unpack(buffer));

        let parent1 = MessageId::unpack(buffer);
        let parent2 = MessageId::unpack(buffer);

        let payload_len = u32::unpack(buffer) as usize;
        let payload = Payload::unpack(buffer);
        assert_eq!(payload_len, payload.len_bytes());

        let nonce = u64::unpack(buffer);

        Self {
            parent1,
            parent2,
            payload,
            nonce,
        }
    }
}

impl Vertex for Message {
    type Id = MessageId;

    fn trunk(&self) -> &Self::Id {
        &self.parent1
    }

    fn branch(&self) -> &Self::Id {
        &self.parent2
    }
}

#[derive(Default)]
pub struct MessageBuilder {
    parent1: Option<MessageId>,
    parent2: Option<MessageId>,
    payload: Option<Payload>,
}

impl MessageBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn parent1(mut self, parent1: MessageId) -> Self {
        self.parent1 = Some(parent1);
        self
    }

    pub fn parent2(mut self, parent2: MessageId) -> Self {
        self.parent2 = Some(parent2);
        self
    }

    pub fn payload(mut self, payload: Payload) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn build(self) -> Result<Message, Error> {
        Ok(Message {
            parent1: self.parent1.ok_or(Error::MissingField("parent1"))?,
            parent2: self.parent2.ok_or(Error::MissingField("parent2"))?,
            payload: self.payload.ok_or(Error::MissingField("payload"))?,
            // TODO PoW
            nonce: 0,
        })
    }
}
