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
    atomic::{payload::Payload, Error, Hash},
    Vertex,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    parent1: Hash,
    parent2: Hash,
    payload: Payload,
    nonce: u64,
}

impl Message {
    pub fn builder() -> MessageBuilder {
        MessageBuilder::new()
    }

    pub fn parent1(&self) -> &Hash {
        &self.parent1
    }

    pub fn parent2(&self) -> &Hash {
        &self.parent2
    }

    pub fn payload(&self) -> &Payload {
        &self.payload
    }

    pub fn nonce(&self) -> u64 {
        self.nonce
    }
}

impl Vertex for Message {
    type Hash = Hash;

    fn trunk(&self) -> &Self::Hash {
        &self.parent1
    }

    fn branch(&self) -> &Self::Hash {
        &self.parent2
    }
}

#[derive(Debug, Default)]
pub struct MessageBuilder {
    parent1: Option<Hash>,
    parent2: Option<Hash>,
    payload: Option<Payload>,
}

impl MessageBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn parent1(mut self, parent1: Hash) -> Self {
        self.parent1 = Some(parent1);
        self
    }

    pub fn parent2(mut self, parent2: Hash) -> Self {
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
