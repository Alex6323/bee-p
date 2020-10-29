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
use bee_message::{
    payload::{
        indexation::HashedIndex,
        transaction::{Transaction, TransactionId},
    },
    Message, MessageId,
};
use bee_storage::access::{Batch, BatchBuilder, CommitBatch};

use blake2::Blake2b;
use rocksdb::{WriteBatch, WriteOptions};

pub struct StorageBatch<'a> {
    storage: &'a Storage,
    batch: WriteBatch,
    // TODO use them to avoid allocating during a same batch
    key_buf: Vec<u8>,
    value_buf: Vec<u8>,
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

impl<'a> Batch<'a> for Storage {
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

impl<'a> BatchBuilder<'a, Storage, MessageId, Message> for StorageBatch<'a> {
    type Error = OpError;

    fn try_insert(mut self, message_id: &MessageId, message: &Message) -> Result<Self, (Self, Self::Error)> {
        let message_id_to_message = self.storage.inner.cf_handle(MESSAGE_ID_TO_MESSAGE).unwrap();

        let mut message_buf = Vec::with_capacity(message.packed_len());
        message.pack(&mut message_buf).unwrap();

        self.batch.put_cf(&message_id_to_message, message_id, message_buf);

        Ok(self)
    }

    fn try_delete(mut self, message_id: &MessageId) -> Result<Self, (Self, Self::Error)> {
        let message_id_to_message = self.storage.inner.cf_handle(MESSAGE_ID_TO_MESSAGE).unwrap();

        self.batch.delete_cf(&message_id_to_message, message_id);

        Ok(self)
    }
}

impl<'a> BatchBuilder<'a, Storage, TransactionId, Transaction> for StorageBatch<'a> {
    type Error = OpError;

    fn try_insert(
        mut self,
        transaction_id: &TransactionId,
        transaction: &Transaction,
    ) -> Result<Self, (Self, Self::Error)> {
        let transaction_id_to_transaction = self.storage.inner.cf_handle(TRANSACTION_ID_TO_TRANSACTION).unwrap();

        let mut transaction_buf = Vec::with_capacity(transaction.packed_len());
        transaction.pack(&mut transaction_buf).unwrap();

        self.batch
            .put_cf(&transaction_id_to_transaction, transaction_id, transaction_buf);

        Ok(self)
    }

    fn try_delete(mut self, transaction_id: &TransactionId) -> Result<Self, (Self, Self::Error)> {
        let transaction_id_to_transaction = self.storage.inner.cf_handle(TRANSACTION_ID_TO_TRANSACTION).unwrap();

        self.batch.delete_cf(&transaction_id_to_transaction, transaction_id);

        Ok(self)
    }
}

impl<'a> BatchBuilder<'a, Storage, (HashedIndex<Blake2b>, MessageId), ()> for StorageBatch<'a> {
    type Error = OpError;

    fn try_insert(
        mut self,
        (index, message_id): &(HashedIndex<Blake2b>, MessageId),
        (): &(),
    ) -> Result<Self, (Self, Self::Error)> {
        let payload_index_to_message_id = self.storage.inner.cf_handle(PAYLOAD_INDEX_TO_MESSAGE_ID).unwrap();

        let mut key = index.as_ref().to_vec();
        key.extend_from_slice(message_id.as_ref());

        self.batch.put_cf(&payload_index_to_message_id, key, []);

        Ok(self)
    }

    fn try_delete(
        mut self,
        (index, message_id): &(HashedIndex<Blake2b>, MessageId),
    ) -> Result<Self, (Self, Self::Error)> {
        let payload_index_to_message_id = self.storage.inner.cf_handle(PAYLOAD_INDEX_TO_MESSAGE_ID).unwrap();

        let mut key = index.as_ref().to_vec();
        key.extend_from_slice(message_id.as_ref());

        self.batch.delete_cf(&payload_index_to_message_id, key);

        Ok(self)
    }
}

impl<'a> BatchBuilder<'a, Storage, (HashedIndex<Blake2b>, TransactionId), ()> for StorageBatch<'a> {
    type Error = OpError;

    fn try_insert(
        mut self,
        (index, transaction_id): &(HashedIndex<Blake2b>, TransactionId),
        (): &(),
    ) -> Result<Self, (Self, Self::Error)> {
        let payload_index_to_transaction_id = self.storage.inner.cf_handle(PAYLOAD_INDEX_TO_TRANSACTION_ID).unwrap();

        let mut key = index.as_ref().to_vec();
        key.extend_from_slice(transaction_id.as_ref());

        self.batch.put_cf(&payload_index_to_transaction_id, key, []);

        Ok(self)
    }

    fn try_delete(
        mut self,
        (index, transaction_id): &(HashedIndex<Blake2b>, TransactionId),
    ) -> Result<Self, (Self, Self::Error)> {
        let payload_index_to_transaction_id = self.storage.inner.cf_handle(PAYLOAD_INDEX_TO_TRANSACTION_ID).unwrap();

        let mut key = index.as_ref().to_vec();
        key.extend_from_slice(transaction_id.as_ref());

        self.batch.delete_cf(&payload_index_to_transaction_id, key);

        Ok(self)
    }
}
