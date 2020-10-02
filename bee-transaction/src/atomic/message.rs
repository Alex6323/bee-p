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

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
    tips: Option<(Hash, Hash)>,
    payload: Option<Payload>,
}

impl MessageBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn tips(mut self, tips: (Hash, Hash)) -> Self {
        self.tips = Some(tips);
        self
    }

    pub fn payload(mut self, payload: Payload) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn build(self) -> Result<Message, Error> {
        let tips = match self.tips {
            Some(t) => t,
            None => return Err(Error::MissingParameter),
        };

        let payload = match self.payload {
            Some(p) => p,
            None => return Err(Error::MissingParameter),
        };

        Ok(Message {
            parent1: tips.0,
            parent2: tips.1,
            payload,
            nonce: 0, // TODO PoW
        })
    }
}
