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

use crate::storage::*;

use bee_common::packable::Packable;
use bee_ledger::{output::Output, spent::Spent, unspent::Unspent};
use bee_message::{
    payload::{indexation::HashedIndex, transaction::OutputId},
    Message, MessageId,
};
use bee_storage::access::Batch;

use blake2::Blake2b;
use rocksdb::{WriteBatch, WriteOptions};

#[derive(Default)]
pub struct StorageBatch {
    batch: WriteBatch,
    // TODO use them to avoid allocating during a same batch
    key_buf: Vec<u8>,
    value_buf: Vec<u8>,
}

#[async_trait::async_trait]
impl Batch<(MessageId, MessageId), ()> for Storage {
    type BatchBuilder = StorageBatch;

    fn batch_insert(
        &self,
        batch: &mut StorageBatch,
        (parent, child): &(MessageId, MessageId),
        (): &(),
    ) -> Result<(), <Self as Backend>::Error> {
        let cf_message_id_to_message_id = self.inner.cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID).unwrap();

        let mut key = parent.as_ref().to_vec();
        key.extend_from_slice(child.as_ref());

        batch.batch.put_cf(&cf_message_id_to_message_id, key, []);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut StorageBatch,
        (parent, child): &(MessageId, MessageId),
    ) -> Result<(), <Self as Backend>::Error> {
        let cf_message_id_to_message_id = self.inner.cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID).unwrap();

        let mut key = parent.as_ref().to_vec();
        key.extend_from_slice(child.as_ref());

        batch.batch.delete_cf(&cf_message_id_to_message_id, key);

        Ok(())
    }

    async fn commit_batch(&self, batch: Self::BatchBuilder, durability: bool) -> Result<(), <Self as Backend>::Error> {
        let mut write_options = WriteOptions::default();
        write_options.set_sync(false);
        write_options.disable_wal(!durability);
        self.inner.write_opt(batch.batch, &write_options)?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Batch<MessageId, Message> for Storage {
    type BatchBuilder = StorageBatch;

    fn batch_insert(
        &self,
        batch: &mut Self::BatchBuilder,
        message_id: &MessageId,
        message: &Message,
    ) -> Result<(), <Self as Backend>::Error> {
        let cf_message_id_to_message = self.inner.cf_handle(CF_MESSAGE_ID_TO_MESSAGE).unwrap();

        let mut message_buf = Vec::with_capacity(message.packed_len());
        message.pack(&mut message_buf).unwrap();

        batch.batch.put_cf(&cf_message_id_to_message, message_id, message_buf);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::BatchBuilder,
        message_id: &MessageId,
    ) -> Result<(), <Self as Backend>::Error> {
        let cf_message_id_to_message = self.inner.cf_handle(CF_MESSAGE_ID_TO_MESSAGE).unwrap();

        batch.batch.delete_cf(&cf_message_id_to_message, message_id);

        Ok(())
    }

    async fn commit_batch(&self, batch: Self::BatchBuilder, durability: bool) -> Result<(), <Self as Backend>::Error> {
        let mut write_options = WriteOptions::default();
        write_options.set_sync(false);
        write_options.disable_wal(!durability);
        self.inner.write_opt(batch.batch, &write_options)?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Batch<(HashedIndex<Blake2b>, MessageId), ()> for Storage {
    type BatchBuilder = StorageBatch;

    fn batch_insert(
        &self,
        batch: &mut Self::BatchBuilder,
        (index, message_id): &(HashedIndex<Blake2b>, MessageId),
        (): &(),
    ) -> Result<(), <Self as Backend>::Error> {
        let cf_payload_index_to_message_id = self.inner.cf_handle(CF_PAYLOAD_INDEX_TO_MESSAGE_ID).unwrap();

        let mut key = index.as_ref().to_vec();
        key.extend_from_slice(message_id.as_ref());

        batch.batch.put_cf(&cf_payload_index_to_message_id, key, []);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::BatchBuilder,
        (index, message_id): &(HashedIndex<Blake2b>, MessageId),
    ) -> Result<(), <Self as Backend>::Error> {
        let cf_payload_index_to_message_id = self.inner.cf_handle(CF_PAYLOAD_INDEX_TO_MESSAGE_ID).unwrap();

        let mut key = index.as_ref().to_vec();
        key.extend_from_slice(message_id.as_ref());

        batch.batch.delete_cf(&cf_payload_index_to_message_id, key);

        Ok(())
    }

    async fn commit_batch(&self, batch: Self::BatchBuilder, durability: bool) -> Result<(), <Self as Backend>::Error> {
        let mut write_options = WriteOptions::default();
        write_options.set_sync(false);
        write_options.disable_wal(!durability);
        self.inner.write_opt(batch.batch, &write_options)?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Batch<OutputId, Output> for Storage {
    type BatchBuilder = StorageBatch;

    fn batch_insert(
        &self,
        batch: &mut Self::BatchBuilder,
        output_id: &OutputId,
        output: &Output,
    ) -> Result<(), <Self as Backend>::Error> {
        let cf_output_id_to_output = self.inner.cf_handle(CF_OUTPUT_ID_TO_OUTPUT).unwrap();

        let mut output_id_buf = Vec::with_capacity(output_id.packed_len());
        // Packing to bytes can't fail.
        output_id.pack(&mut output_id_buf).unwrap();
        let mut output_buf = Vec::with_capacity(output.packed_len());
        // Packing to bytes can't fail.
        output.pack(&mut output_buf).unwrap();

        batch.batch.put_cf(&cf_output_id_to_output, output_id_buf, output_buf);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::BatchBuilder,
        output_id: &OutputId,
    ) -> Result<(), <Self as Backend>::Error> {
        let cf_output_id_to_output = self.inner.cf_handle(CF_OUTPUT_ID_TO_OUTPUT).unwrap();

        let mut output_id_buf = Vec::with_capacity(output_id.packed_len());
        // Packing to bytes can't fail.
        output_id.pack(&mut output_id_buf).unwrap();

        batch.batch.delete_cf(&cf_output_id_to_output, output_id_buf);

        Ok(())
    }

    async fn commit_batch(&self, batch: Self::BatchBuilder, durability: bool) -> Result<(), <Self as Backend>::Error> {
        let mut write_options = WriteOptions::default();
        write_options.set_sync(false);
        write_options.disable_wal(!durability);
        self.inner.write_opt(batch.batch, &write_options)?;

        Ok(())
    }
}
#[async_trait::async_trait]
impl Batch<OutputId, Spent> for Storage {
    type BatchBuilder = StorageBatch;

    fn batch_insert(
        &self,
        batch: &mut Self::BatchBuilder,
        output_id: &OutputId,
        spent: &Spent,
    ) -> Result<(), <Self as Backend>::Error> {
        let cf_output_id_to_spent = self.inner.cf_handle(CF_OUTPUT_ID_TO_SPENT).unwrap();

        let mut output_id_buf = Vec::with_capacity(output_id.packed_len());
        // Packing to bytes can't fail.
        output_id.pack(&mut output_id_buf).unwrap();
        let mut spent_buf = Vec::with_capacity(spent.packed_len());
        // Packing to bytes can't fail.
        spent.pack(&mut spent_buf).unwrap();

        batch.batch.put_cf(&cf_output_id_to_spent, output_id_buf, spent_buf);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::BatchBuilder,
        output_id: &OutputId,
    ) -> Result<(), <Self as Backend>::Error> {
        let cf_output_id_to_spent = self.inner.cf_handle(CF_OUTPUT_ID_TO_SPENT).unwrap();

        let mut output_id_buf = Vec::with_capacity(output_id.packed_len());
        // Packing to bytes can't fail.
        output_id.pack(&mut output_id_buf).unwrap();

        batch.batch.delete_cf(&cf_output_id_to_spent, output_id_buf);

        Ok(())
    }

    async fn commit_batch(&self, batch: Self::BatchBuilder, durability: bool) -> Result<(), <Self as Backend>::Error> {
        let mut write_options = WriteOptions::default();
        write_options.set_sync(false);
        write_options.disable_wal(!durability);
        self.inner.write_opt(batch.batch, &write_options)?;

        Ok(())
    }
}
#[async_trait::async_trait]
impl Batch<Unspent, ()> for Storage {
    type BatchBuilder = StorageBatch;

    fn batch_insert(&self, batch: &mut Self::BatchBuilder, unspent: &Unspent, (): &()) -> Result<(), Self::Error> {
        let cf_output_id_unspent = self.inner.cf_handle(CF_OUTPUT_ID_UNSPENT).unwrap();

        let mut unspent_buf = Vec::with_capacity(unspent.packed_len());
        // Packing to bytes can't fail.
        unspent.pack(&mut unspent_buf).unwrap();

        batch.batch.put_cf(&cf_output_id_unspent, unspent_buf, []);

        Ok(())
    }

    fn batch_delete(&self, batch: &mut Self::BatchBuilder, unspent: &Unspent) -> Result<(), Self::Error> {
        let cf_output_id_unspent = self.inner.cf_handle(CF_OUTPUT_ID_UNSPENT).unwrap();

        let mut unspent_buf = Vec::with_capacity(unspent.packed_len());
        // Packing to bytes can't fail.
        unspent.pack(&mut unspent_buf).unwrap();

        batch.batch.delete_cf(&cf_output_id_unspent, unspent_buf);

        Ok(())
    }

    async fn commit_batch(&self, batch: Self::BatchBuilder, durability: bool) -> Result<(), <Self as Backend>::Error> {
        let mut write_options = WriteOptions::default();
        write_options.set_sync(false);
        write_options.disable_wal(!durability);
        self.inner.write_opt(batch.batch, &write_options)?;

        Ok(())
    }
}
