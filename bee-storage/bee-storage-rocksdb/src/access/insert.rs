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
use bee_message::{payload::indexation::IndexHash, Message, MessageId};
use bee_protocol::{tangle::MessageMetadata, MilestoneIndex};
use bee_storage::{access::Insert, persistable::Persistable};

use crate::{access::OpError, storage::*};

use digest::Digest;

#[async_trait::async_trait]
impl Insert<Hash, MessageMetadata> for Storage {
    type Error = OpError;
    async fn insert(&self, hash: &Hash, tx_metadata: &MessageMetadata) -> Result<(), Self::Error> {
        let hash_to_metadata = self.inner.cf_handle(TRANSACTION_HASH_TO_METADATA).unwrap();
        let mut hash_buf = Vec::new();
        hash.write_to(&mut hash_buf);
        let mut metadata_buf = Vec::new();
        tx_metadata.write_to(&mut metadata_buf);
        self.inner
            .put_cf(&hash_to_metadata, hash_buf.as_slice(), metadata_buf.as_slice())?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<Hash, MilestoneIndex> for Storage {
    type Error = OpError;
    async fn insert(&self, hash: &Hash, milestone_index: &MilestoneIndex) -> Result<(), Self::Error> {
        let ms_hash_to_ms_index = self.inner.cf_handle(MILESTONE_HASH_TO_INDEX).unwrap();
        let mut hash_buf = Vec::new();
        hash.write_to(&mut hash_buf);
        let mut index_buf = Vec::new();
        milestone_index.write_to(&mut index_buf);
        self.inner
            .put_cf(&ms_hash_to_ms_index, hash_buf.as_slice(), index_buf.as_slice())?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<MessageId, Message> for Storage {
    type Error = OpError;
    async fn insert(&self, message_id: &MessageId, message: &Message) -> Result<(), Self::Error> {
        let message_id_to_message = self.inner.cf_handle(MESSAGE_ID_TO_MESSAGE).unwrap();

        let mut message_buf = Vec::new();
        message.pack(&mut message_buf).unwrap();

        self.inner
            .put_cf(&message_id_to_message, message_id.as_ref(), message_buf.as_slice())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl<D: Digest> Insert<IndexHash<D>, MessageId> for Storage {
    type Error = OpError;
    async fn insert(&self, index: &IndexHash<D>, message_id: &MessageId) -> Result<(), Self::Error> {
        let payload_index_to_message_id = self.inner.cf_handle(PAYLOAD_INDEX_TO_MESSAGE_ID).unwrap();

        let mut entry_buf = index.as_ref().to_vec();
        entry_buf.extend_from_slice(message_id.as_ref());

        self.inner
            .put_cf(&payload_index_to_message_id, entry_buf.as_slice(), &[])?;

        Ok(())
    }
}
