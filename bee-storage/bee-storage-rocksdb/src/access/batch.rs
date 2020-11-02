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

use crate::{access::OpError, storage::*};

use bee_common_ext::packable::Packable;
use bee_ledger::spent::Spent;
use bee_message::{
    payload::{indexation::HashedIndex, transaction::OutputId},
    Message, MessageId,
};
use bee_storage::access::{BatchBuilder, BeginBatch, CommitBatch};

use blake2::Blake2b;
use rocksdb::{WriteBatch, WriteOptions};

pub struct StorageBatch<'a> {
    storage: &'a Storage,
    batch: WriteBatch,
    // TODO use them to avoid allocating during a same batch
    key_buf: Vec<u8>,
    value_buf: Vec<u8>,
}

impl<'a> BeginBatch<'a> for Storage {
    type BatchBuilder = StorageBatch<'a>;

    fn begin_batch(&'a self) -> Self::BatchBuilder {
        Self::BatchBuilder {
            storage: self,
            batch: WriteBatch::default(),
            key_buf: Vec::new(),
            value_buf: Vec::new(),
        }
    }
}

#[async_trait::async_trait]
impl<'a> CommitBatch for StorageBatch<'a> {
    type Error = OpError;

    async fn commit_batch(self, durability: bool) -> Result<(), Self::Error> {
        let mut write_options = WriteOptions::default();
        write_options.set_sync(false);
        write_options.disable_wal(!durability);
        self.storage.inner.write_opt(self.batch, &write_options)?;

        Ok(())
    }
}

impl<'a> BatchBuilder<'a, Storage, (MessageId, MessageId), ()> for StorageBatch<'a> {
    type Error = OpError;

    fn try_insert(&mut self, (parent, child): &(MessageId, MessageId), (): &()) -> Result<(), Self::Error> {
        let cf_message_id_to_message_id = self.storage.inner.cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID).unwrap();

        let mut key = parent.as_ref().to_vec();
        key.extend_from_slice(child.as_ref());

        self.batch.put_cf(&cf_message_id_to_message_id, key, []);

        Ok(())
    }

    fn try_delete(&mut self, (parent, child): &(MessageId, MessageId)) -> Result<(), Self::Error> {
        let cf_message_id_to_message_id = self.storage.inner.cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID).unwrap();

        let mut key = parent.as_ref().to_vec();
        key.extend_from_slice(child.as_ref());

        self.batch.delete_cf(&cf_message_id_to_message_id, key);

        Ok(())
    }
}

impl<'a> BatchBuilder<'a, Storage, MessageId, Message> for StorageBatch<'a> {
    type Error = OpError;

    fn try_insert(&mut self, message_id: &MessageId, message: &Message) -> Result<(), Self::Error> {
        let cf_message_id_to_message = self.storage.inner.cf_handle(CF_MESSAGE_ID_TO_MESSAGE).unwrap();

        let mut message_buf = Vec::with_capacity(message.packed_len());
        message.pack(&mut message_buf).unwrap();

        self.batch.put_cf(&cf_message_id_to_message, message_id, message_buf);

        Ok(())
    }

    fn try_delete(&mut self, message_id: &MessageId) -> Result<(), Self::Error> {
        let cf_message_id_to_message = self.storage.inner.cf_handle(CF_MESSAGE_ID_TO_MESSAGE).unwrap();

        self.batch.delete_cf(&cf_message_id_to_message, message_id);

        Ok(())
    }
}

impl<'a> BatchBuilder<'a, Storage, (HashedIndex<Blake2b>, MessageId), ()> for StorageBatch<'a> {
    type Error = OpError;

    fn try_insert(
        &mut self,
        (index, message_id): &(HashedIndex<Blake2b>, MessageId),
        (): &(),
    ) -> Result<(), Self::Error> {
        let cf_payload_index_to_message_id = self.storage.inner.cf_handle(CF_PAYLOAD_INDEX_TO_MESSAGE_ID).unwrap();

        let mut key = index.as_ref().to_vec();
        key.extend_from_slice(message_id.as_ref());

        self.batch.put_cf(&cf_payload_index_to_message_id, key, []);

        Ok(())
    }

    fn try_delete(&mut self, (index, message_id): &(HashedIndex<Blake2b>, MessageId)) -> Result<(), Self::Error> {
        let cf_payload_index_to_message_id = self.storage.inner.cf_handle(CF_PAYLOAD_INDEX_TO_MESSAGE_ID).unwrap();

        let mut key = index.as_ref().to_vec();
        key.extend_from_slice(message_id.as_ref());

        self.batch.delete_cf(&cf_payload_index_to_message_id, key);

        Ok(())
    }
}

impl<'a> BatchBuilder<'a, Storage, OutputId, Spent> for StorageBatch<'a> {
    type Error = OpError;

    fn try_insert(&mut self, output_id: &OutputId, spent: &Spent) -> Result<(), Self::Error> {
        let cf_output_id_to_spent = self.storage.inner.cf_handle(CF_OUTPUT_ID_TO_SPENT).unwrap();

        let mut output_id_buf = Vec::with_capacity(output_id.packed_len());
        // Packing to bytes can't fail.
        output_id.pack(&mut output_id_buf).unwrap();
        let mut spent_buf = Vec::with_capacity(spent.packed_len());
        // Packing to bytes can't fail.
        spent.pack(&mut spent_buf).unwrap();

        self.batch.put_cf(&cf_output_id_to_spent, output_id_buf, spent_buf);

        Ok(())
    }

    fn try_delete(&mut self, output_id: &OutputId) -> Result<(), Self::Error> {
        let cf_output_id_to_spent = self.storage.inner.cf_handle(CF_OUTPUT_ID_TO_SPENT).unwrap();

        let mut output_id_buf = Vec::with_capacity(output_id.packed_len());
        // Packing to bytes can't fail.
        output_id.pack(&mut output_id_buf).unwrap();

        self.batch.delete_cf(&cf_output_id_to_spent, output_id_buf);

        Ok(())
    }
}
