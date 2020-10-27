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

use crate::MessageRef;

use bee_message::{Message, MessageId};

use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct Vertex<T>
where
    T: Clone,
{
    message: MessageRef,
    metadata: T,
}

impl<T> Vertex<T>
where
    T: Clone,
{
    pub fn new(message: Message, metadata: T) -> Self {
        Self {
            message: MessageRef(Arc::new(message)),
            metadata,
        }
    }

    pub fn parent1(&self) -> &MessageId {
        self.message.parent1()
    }

    pub fn parent2(&self) -> &MessageId {
        self.message.parent2()
    }

    pub fn message(&self) -> &MessageRef {
        &self.message
    }

    pub fn metadata(&self) -> &T {
        &self.metadata
    }

    pub fn metadata_mut(&mut self) -> &mut T {
        &mut self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bee_test::transaction::create_random_tx;

    #[test]
    fn create_new_vertex() {
        let (_, tx) = create_random_tx();
        let metadata = 0b0000_0001u8;

        let vtx = Vertex::new(tx.clone(), metadata);

        assert_eq!(tx.parent1(), vtx.parent1());
        assert_eq!(tx.parent2(), vtx.parent2());
        assert_eq!(tx, **vtx.message());
        assert_eq!(metadata, *vtx.metadata());
    }

    #[test]
    fn update_vertex_meta() {
        let (_, tx) = create_random_tx();

        let mut vtx = Vertex::new(tx, 0b0000_0001u8);
        *vtx.metadata_mut() = 0b1111_1110u8;

        assert_eq!(0b1111_1110u8, *vtx.metadata());
    }
}
