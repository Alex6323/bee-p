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

use crate::{payload::Payload, Error, MessageId, Vertex};

use bee_common_ext::packable::{Error as PackableError, Packable, Read, Write};

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
    fn packed_len(&self) -> usize {
        1u8.packed_len()
            + self.parent1.packed_len()
            + self.parent2.packed_len()
            + 0u32.packed_len()
            + self.payload.packed_len()
            + 0u64.packed_len()
    }

    fn pack<W: Write>(&self, buf: &mut W) -> Result<(), PackableError> {
        1u8.pack(buf)?;

        self.parent1.pack(buf)?;
        self.parent2.pack(buf)?;

        (self.payload.packed_len() as u32).pack(buf)?;
        self.payload.pack(buf)?;

        self.nonce.pack(buf)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(buf: &mut R) -> Result<Self, PackableError>
    where
        Self: Sized,
    {
        let version = u8::unpack(buf)?;

        if version != 1u8 {
            return Err(PackableError::InvalidVersion(1, version));
        }

        let parent1 = MessageId::unpack(buf)?;
        let parent2 = MessageId::unpack(buf)?;

        let payload_len = u32::unpack(buf)? as usize;
        let payload = Payload::unpack(buf)?;

        if payload_len != payload.packed_len() {
            return Err(PackableError::InvalidAnnouncedLength(payload_len, payload.packed_len()));
        }

        let nonce = u64::unpack(buf)?;

        Ok(Self {
            parent1,
            parent2,
            payload,
            nonce,
        })
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

// TODO generic over PoW provider
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

    pub fn with_parent1(mut self, parent1: MessageId) -> Self {
        self.parent1 = Some(parent1);
        self
    }

    pub fn with_parent2(mut self, parent2: MessageId) -> Self {
        self.parent2 = Some(parent2);
        self
    }

    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn finish(self) -> Result<Message, Error> {
        Ok(Message {
            parent1: self.parent1.ok_or(Error::MissingField("parent1"))?,
            parent2: self.parent2.ok_or(Error::MissingField("parent2"))?,
            payload: self.payload.ok_or(Error::MissingField("payload"))?,
            nonce: 0,
        })
    }
}
