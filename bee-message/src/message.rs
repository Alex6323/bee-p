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

use crate::{payload::Payload, Error, MessageId, Vertex, MESSAGE_ID_LENGTH};

use bee_common_ext::packable::{Packable, Read, Write};

use blake2::{Blake2b, Digest};
use serde::{Deserialize, Serialize};

use core::convert::TryInto;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
    parent1: MessageId,
    parent2: MessageId,
    payload: Option<Payload>,
    nonce: u64,
}

impl Message {
    pub fn builder() -> MessageBuilder {
        MessageBuilder::new()
    }

    pub fn id(&self) -> MessageId {
        let mut bytes = Vec::with_capacity(self.packed_len());
        let mut hasher = Blake2b::new();

        // Packing to bytes can't fail.
        self.pack(&mut bytes).unwrap();
        hasher.update(&bytes);

        // We know for sure the bytes have the right size.
        MessageId::new(hasher.finalize()[0..MESSAGE_ID_LENGTH].try_into().unwrap())
    }

    pub fn parent1(&self) -> &MessageId {
        &self.parent1
    }

    pub fn parent2(&self) -> &MessageId {
        &self.parent2
    }

    pub fn payload(&self) -> &Option<Payload> {
        &self.payload
    }

    pub fn nonce(&self) -> u64 {
        self.nonce
    }
}

impl Packable for Message {
    type Error = Error;

    fn packed_len(&self) -> usize {
        1u8.packed_len()
            + self.parent1.packed_len()
            + self.parent2.packed_len()
            + 0u32.packed_len()
            + if let Some(ref payload) = self.payload {
                payload.packed_len()
            } else {
                0
            }
            + 0u64.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        1u8.pack(writer)?;

        self.parent1.pack(writer)?;
        self.parent2.pack(writer)?;

        if let Some(ref payload) = self.payload {
            (payload.packed_len() as u32).pack(writer)?;
            payload.pack(writer)?;
        } else {
            0u32.pack(writer)?;
        }

        self.nonce.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let version = u8::unpack(reader)?;

        if version != 1u8 {
            return Err(Self::Error::InvalidVersion(1, version));
        }

        let parent1 = MessageId::unpack(reader)?;
        let parent2 = MessageId::unpack(reader)?;

        let payload_len = u32::unpack(reader)? as usize;
        let payload = if payload_len != 0 {
            let payload = Payload::unpack(reader)?;
            if payload_len != payload.packed_len() {
                return Err(Self::Error::InvalidAnnouncedLength(payload_len, payload.packed_len()));
            }
            Some(payload)
        } else {
            None
        };

        let nonce = u64::unpack(reader)?;

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

    fn parent1(&self) -> &Self::Id {
        &self.parent1
    }

    fn parent2(&self) -> &Self::Id {
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
            payload: self.payload,
            nonce: 0,
        })
    }
}
