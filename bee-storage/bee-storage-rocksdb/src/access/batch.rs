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
    payload::{
        indexation::HashedIndex,
        transaction::{Ed25519Address, OutputId},
    },
    Message, MessageId,
};
use bee_protocol::tangle::MessageMetadata;
use bee_storage::access::{Batch, BatchBuilder};

use rocksdb::{WriteBatch, WriteOptions};

#[derive(Default)]
pub struct StorageBatch {
    inner: WriteBatch,
    key_buf: Vec<u8>,
    value_buf: Vec<u8>,
}

#[async_trait::async_trait]
impl BatchBuilder for Storage {
    type Batch = StorageBatch;

    async fn batch_commit(&self, batch: Self::Batch, durability: bool) -> Result<(), <Self as Backend>::Error> {
        let mut write_options = WriteOptions::default();
        write_options.set_sync(false);
        write_options.disable_wal(!durability);
        self.inner.write_opt(batch.inner, &write_options)?;

        Ok(())
    }
}

impl Batch<MessageId, Message> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        message_id: &MessageId,
        message: &Message,
    ) -> Result<(), <Self as Backend>::Error> {
        let cf_message_id_to_message = self.inner.cf_handle(CF_MESSAGE_ID_TO_MESSAGE).unwrap();

        batch.value_buf.clear();
        // Packing to bytes can't fail.
        message.pack(&mut batch.value_buf).unwrap();

        batch
            .inner
            .put_cf(&cf_message_id_to_message, message_id, &batch.value_buf);

        Ok(())
    }

    fn batch_delete(&self, batch: &mut Self::Batch, message_id: &MessageId) -> Result<(), <Self as Backend>::Error> {
        let cf_message_id_to_message = self.inner.cf_handle(CF_MESSAGE_ID_TO_MESSAGE).unwrap();

        batch.inner.delete_cf(&cf_message_id_to_message, message_id);

        Ok(())
    }
}

impl Batch<MessageId, MessageMetadata> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        message_id: &MessageId,
        metadata: &MessageMetadata,
    ) -> Result<(), <Self as Backend>::Error> {
        let cf_message_id_to_metadata = self.inner.cf_handle(CF_MESSAGE_ID_TO_METADATA).unwrap();

        batch.value_buf.clear();
        // Packing to bytes can't fail.
        metadata.pack(&mut batch.value_buf).unwrap();

        batch
            .inner
            .put_cf(&cf_message_id_to_metadata, message_id, &batch.value_buf);

        Ok(())
    }

    fn batch_delete(&self, batch: &mut Self::Batch, message_id: &MessageId) -> Result<(), <Self as Backend>::Error> {
        let cf_message_id_to_metadata = self.inner.cf_handle(CF_MESSAGE_ID_TO_METADATA).unwrap();

        batch.inner.delete_cf(&cf_message_id_to_metadata, message_id);

        Ok(())
    }
}

impl Batch<(MessageId, MessageId), ()> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        (parent, child): &(MessageId, MessageId),
        (): &(),
    ) -> Result<(), <Self as Backend>::Error> {
        let cf_message_id_to_message_id = self.inner.cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID).unwrap();

        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(parent.as_ref());
        batch.key_buf.extend_from_slice(child.as_ref());

        batch.inner.put_cf(&cf_message_id_to_message_id, &batch.key_buf, []);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        (parent, child): &(MessageId, MessageId),
    ) -> Result<(), <Self as Backend>::Error> {
        let cf_message_id_to_message_id = self.inner.cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID).unwrap();

        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(parent.as_ref());
        batch.key_buf.extend_from_slice(child.as_ref());

        batch.inner.delete_cf(&cf_message_id_to_message_id, &batch.key_buf);

        Ok(())
    }
}

impl Batch<(HashedIndex, MessageId), ()> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        (index, message_id): &(HashedIndex, MessageId),
        (): &(),
    ) -> Result<(), <Self as Backend>::Error> {
        let cf_index_to_message_id = self.inner.cf_handle(CF_INDEX_TO_MESSAGE_ID).unwrap();

        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(index.as_ref());
        batch.key_buf.extend_from_slice(message_id.as_ref());

        batch.inner.put_cf(&cf_index_to_message_id, &batch.key_buf, []);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        (index, message_id): &(HashedIndex, MessageId),
    ) -> Result<(), <Self as Backend>::Error> {
        let cf_index_to_message_id = self.inner.cf_handle(CF_INDEX_TO_MESSAGE_ID).unwrap();

        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(index.as_ref());
        batch.key_buf.extend_from_slice(message_id.as_ref());

        batch.inner.delete_cf(&cf_index_to_message_id, &batch.key_buf);

        Ok(())
    }
}

impl Batch<OutputId, Output> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        output_id: &OutputId,
        output: &Output,
    ) -> Result<(), <Self as Backend>::Error> {
        let cf_output_id_to_output = self.inner.cf_handle(CF_OUTPUT_ID_TO_OUTPUT).unwrap();

        batch.key_buf.clear();
        // Packing to bytes can't fail.
        output_id.pack(&mut batch.key_buf).unwrap();
        batch.value_buf.clear();
        // Packing to bytes can't fail.
        output.pack(&mut batch.value_buf).unwrap();

        batch
            .inner
            .put_cf(&cf_output_id_to_output, &batch.key_buf, &batch.value_buf);

        Ok(())
    }

    fn batch_delete(&self, batch: &mut Self::Batch, output_id: &OutputId) -> Result<(), <Self as Backend>::Error> {
        let cf_output_id_to_output = self.inner.cf_handle(CF_OUTPUT_ID_TO_OUTPUT).unwrap();

        batch.key_buf.clear();
        // Packing to bytes can't fail.
        output_id.pack(&mut batch.key_buf).unwrap();

        batch.inner.delete_cf(&cf_output_id_to_output, &batch.key_buf);

        Ok(())
    }
}

impl Batch<OutputId, Spent> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        output_id: &OutputId,
        spent: &Spent,
    ) -> Result<(), <Self as Backend>::Error> {
        let cf_output_id_to_spent = self.inner.cf_handle(CF_OUTPUT_ID_TO_SPENT).unwrap();

        batch.key_buf.clear();
        // Packing to bytes can't fail.
        output_id.pack(&mut batch.key_buf).unwrap();
        batch.value_buf.clear();
        // Packing to bytes can't fail.
        spent.pack(&mut batch.value_buf).unwrap();

        batch
            .inner
            .put_cf(&cf_output_id_to_spent, &batch.key_buf, &batch.value_buf);

        Ok(())
    }

    fn batch_delete(&self, batch: &mut Self::Batch, output_id: &OutputId) -> Result<(), <Self as Backend>::Error> {
        let cf_output_id_to_spent = self.inner.cf_handle(CF_OUTPUT_ID_TO_SPENT).unwrap();

        batch.key_buf.clear();
        // Packing to bytes can't fail.
        output_id.pack(&mut batch.key_buf).unwrap();

        batch.inner.delete_cf(&cf_output_id_to_spent, &batch.key_buf);

        Ok(())
    }
}

impl Batch<Unspent, ()> for Storage {
    fn batch_insert(&self, batch: &mut Self::Batch, unspent: &Unspent, (): &()) -> Result<(), Self::Error> {
        let cf_output_id_unspent = self.inner.cf_handle(CF_OUTPUT_ID_UNSPENT).unwrap();

        batch.key_buf.clear();
        // Packing to bytes can't fail.
        unspent.pack(&mut batch.key_buf).unwrap();

        batch.inner.put_cf(&cf_output_id_unspent, &batch.key_buf, []);

        Ok(())
    }

    fn batch_delete(&self, batch: &mut Self::Batch, unspent: &Unspent) -> Result<(), Self::Error> {
        let cf_output_id_unspent = self.inner.cf_handle(CF_OUTPUT_ID_UNSPENT).unwrap();

        batch.key_buf.clear();
        // Packing to bytes can't fail.
        unspent.pack(&mut batch.key_buf).unwrap();

        batch.inner.delete_cf(&cf_output_id_unspent, &batch.key_buf);

        Ok(())
    }
}

impl Batch<(Ed25519Address, OutputId), ()> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        (address, output_id): &(Ed25519Address, OutputId),
        (): &(),
    ) -> Result<(), <Self as Backend>::Error> {
        let cf_ed25519_address_to_output_id = self.inner.cf_handle(CF_ED25519_ADDRESS_TO_OUTPUT_ID).unwrap();

        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(address.as_ref());
        batch.key_buf.extend_from_slice(&output_id.pack_new());

        batch.inner.put_cf(&cf_ed25519_address_to_output_id, &batch.key_buf, []);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        (address, output_id): &(Ed25519Address, OutputId),
    ) -> Result<(), <Self as Backend>::Error> {
        let cf_ed25519_address_to_output_id = self.inner.cf_handle(CF_ED25519_ADDRESS_TO_OUTPUT_ID).unwrap();

        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(address.as_ref());
        batch.key_buf.extend_from_slice(&output_id.pack_new());

        batch.inner.delete_cf(&cf_ed25519_address_to_output_id, &batch.key_buf);

        Ok(())
    }
}
