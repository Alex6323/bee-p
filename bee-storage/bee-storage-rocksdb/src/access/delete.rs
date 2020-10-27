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

use bee_crypto::ternary::Hash;
use bee_message::{Message, MessageId};
use bee_protocol::{tangle::MessageMetadata, MilestoneIndex};
use bee_storage::{access::Delete, persistable::Persistable};

use crate::{access::OpError, storage::*};

use blake2::{Blake2b, Digest};

#[async_trait::async_trait]
impl Delete<Hash, MessageMetadata> for Storage {
    type Error = OpError;
    async fn delete(&self, hash: &Hash) -> Result<(), Self::Error> {
        let db = &self.inner;
        let hash_to_metadata = db.cf_handle(TRANSACTION_HASH_TO_METADATA).unwrap();
        let mut hash_buf = Vec::new();
        hash.write_to(&mut hash_buf);
        db.delete_cf(&hash_to_metadata, hash_buf.as_slice())?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<Hash, MilestoneIndex> for Storage {
    type Error = OpError;
    async fn delete(&self, hash: &Hash) -> Result<(), Self::Error> {
        let db = &self.inner;
        let ms_hash_to_ms_index = db.cf_handle(MILESTONE_HASH_TO_INDEX).unwrap();
        let mut hash_buf = Vec::new();
        hash.write_to(&mut hash_buf);
        db.delete_cf(&ms_hash_to_ms_index, hash_buf.as_slice())?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<MessageId, Message> for Storage {
    type Error = OpError;
    async fn delete(&self, message_id: &MessageId) -> Result<(), Self::Error> {
        let db = &self.inner;

        let message_id_to_message = db.cf_handle(MESSAGE_ID_TO_MESSAGE).unwrap();

        db.delete_cf(&message_id_to_message, message_id.as_ref())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<(String, MessageId), ()> for Storage {
    type Error = OpError;
    async fn delete(&self, (index, message_id): &(String, MessageId)) -> Result<(), Self::Error> {
        let db = &self.inner;

        let payload_index_to_message_id = db.cf_handle(PAYLOAD_INDEX_TO_MESSAGE_ID).unwrap();

        let mut hasher = Blake2b::new();
        hasher.update(index.as_bytes());
        let mut indexation_buf = hasher.finalize().to_vec();

        indexation_buf.extend_from_slice(message_id.as_ref());

        db.delete_cf(&payload_index_to_message_id, indexation_buf.as_slice())?;

        Ok(())
    }
}
