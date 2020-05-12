// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! MilestoneRequest message of the protocol version 2

use crate::message::Message;

use std::{convert::TryInto, ops::Range};

const INDEX_SIZE: usize = 4;
const CONSTANT_SIZE: usize = INDEX_SIZE;

/// A message to request a milestone.
#[derive(Clone, Default)]
pub(crate) struct MilestoneRequest {
    /// Index of the requested milestone.
    pub(crate) index: u32,
}

impl MilestoneRequest {
    pub(crate) fn new(index: u32) -> Self {
        Self { index }
    }
}

impl Message for MilestoneRequest {
    const ID: u8 = 0x03;

    fn size_range() -> Range<usize> {
        (CONSTANT_SIZE)..(CONSTANT_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut message = Self::default();

        message.index = u32::from_be_bytes(bytes[0..INDEX_SIZE].try_into().expect("Invalid buffer size"));

        message
    }

    fn size(&self) -> usize {
        CONSTANT_SIZE
    }

    fn into_bytes(self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.index.to_be_bytes())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    const INDEX: u32 = 0x81f7df7c;

    #[test]
    fn id() {
        assert_eq!(MilestoneRequest::ID, 3);
    }

    #[test]
    fn size_range() {
        assert_eq!(MilestoneRequest::size_range().contains(&3), false);
        assert_eq!(MilestoneRequest::size_range().contains(&4), true);
        assert_eq!(MilestoneRequest::size_range().contains(&5), false);
    }

    #[test]
    fn size() {
        let message = MilestoneRequest::new(INDEX);

        assert_eq!(message.size(), CONSTANT_SIZE);
    }

    #[test]
    fn into_from() {
        let message_from = MilestoneRequest::new(INDEX);
        let mut bytes = vec![0u8; message_from.size()];
        message_from.into_bytes(&mut bytes);
        let message_to = MilestoneRequest::from_bytes(&bytes);

        assert_eq!(message_to.index, INDEX);
    }
}
