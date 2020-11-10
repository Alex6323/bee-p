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
    bytes::{random_bytes, random_bytes_32},
    string::random_string,
};

use bee_message::{
    payload::{indexation::Indexation, Payload},
    Message, MessageId,
};

use rand::Rng;

pub fn random_message_id() -> MessageId {
    MessageId::new(random_bytes_32())
}

pub fn random_indexation() -> Payload {
    Indexation::new(random_string(32), &random_bytes(64)).unwrap().into()
}

pub fn random_payload() -> Payload {
    // TODO complete with other types
    random_indexation()
}

pub fn random_message_with_parents(parent1: MessageId, parent2: MessageId) -> Message {
    Message::builder()
        .with_parent1(parent1)
        .with_parent2(parent2)
        .with_payload(random_payload())
        .with_nonce(rand::thread_rng().gen::<u64>())
        .finish()
        .unwrap()
}

pub fn random_message() -> Message {
    random_message_with_parents(random_message_id(), random_message_id())
}
