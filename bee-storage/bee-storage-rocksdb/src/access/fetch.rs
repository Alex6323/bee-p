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

use bee_common_ext::packable::Packable;
use bee_crypto::ternary::Hash;
use bee_message::{Message, MessageId};
use bee_protocol::{tangle::MessageMetadata, MilestoneIndex};
use bee_storage::{access::Fetch, persistable::Persistable};

use crate::{access::OpError, storage::*};

#[async_trait::async_trait]
impl Fetch<Hash, MessageMetadata> for Storage {
    type Error = OpError;
    async fn fetch(&self, hash: &Hash) -> Result<Option<MessageMetadata>, OpError>
    where
        Self: Sized,
    {
        let hash_to_metadata = self.inner.cf_handle(TRANSACTION_HASH_TO_METADATA).unwrap();
        let mut hash_buf: Vec<u8> = Vec::new();
        hash.write_to(&mut hash_buf);
        if let Some(res) = self.inner.get_cf(&hash_to_metadata, hash_buf.as_slice())? {
            let transaction_metadata: MessageMetadata = MessageMetadata::read_from(res.as_slice());
            Ok(Some(transaction_metadata))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl Fetch<Hash, MilestoneIndex> for Storage {
    type Error = OpError;
    async fn fetch(&self, hash: &Hash) -> Result<Option<MilestoneIndex>, Self::Error>
    where
        Self: Sized,
    {
        let ms_hash_to_ms_index = self.inner.cf_handle(MILESTONE_HASH_TO_INDEX).unwrap();
        let mut hash_buf: Vec<u8> = Vec::new();
        hash.write_to(&mut hash_buf);
        if let Some(res) = self.inner.get_cf(&ms_hash_to_ms_index, hash_buf.as_slice())? {
            let ms_index: MilestoneIndex = MilestoneIndex::read_from(res.as_slice());
            Ok(Some(ms_index))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl Fetch<MessageId, Message> for Storage {
    type Error = OpError;
    async fn fetch(&self, message_id: &MessageId) -> Result<Option<Message>, Self::Error>
    where
        Self: Sized,
    {
        let message_id_to_message = self.inner.cf_handle(MESSAGE_ID_TO_MESSAGE).unwrap();

        if let Some(res) = self.inner.get_cf(&message_id_to_message, message_id.as_ref())? {
            let message = Message::unpack(&mut res.as_slice()).unwrap();

            Ok(Some(message))
        } else {
            Ok(None)
        }
    }
}
