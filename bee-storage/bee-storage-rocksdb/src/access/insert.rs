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
use bee_storage::access::Insert;

#[async_trait::async_trait]
impl Insert<MessageId, Message> for Storage {
    async fn insert(&self, message_id: &MessageId, message: &Message) -> Result<(), <Self as Backend>::Error> {
        let cf_message_id_to_message = self.inner.cf_handle(CF_MESSAGE_ID_TO_MESSAGE).unwrap();

        // Packing to bytes can't fail.
        self.inner
            .put_cf(&cf_message_id_to_message, message_id, message.pack_new().unwrap())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<(MessageId, MessageId), ()> for Storage {
    async fn insert(&self, (parent, child): &(MessageId, MessageId), (): &()) -> Result<(), <Self as Backend>::Error> {
        let cf_message_id_to_message_id = self.inner.cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID).unwrap();

        let mut key = parent.as_ref().to_vec();
        key.extend_from_slice(child.as_ref());

        self.inner.put_cf(&cf_message_id_to_message_id, key, [])?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<(HashedIndex, MessageId), ()> for Storage {
    async fn insert(
        &self,
        (index, message_id): &(HashedIndex, MessageId),
        (): &(),
    ) -> Result<(), <Self as Backend>::Error> {
        let cf_payload_index_to_message_id = self.inner.cf_handle(CF_PAYLOAD_INDEX_TO_MESSAGE_ID).unwrap();

        let mut key = index.as_ref().to_vec();
        key.extend_from_slice(message_id.as_ref());

        self.inner.put_cf(&cf_payload_index_to_message_id, key, [])?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<OutputId, Output> for Storage {
    async fn insert(&self, output_id: &OutputId, output: &Output) -> Result<(), <Self as Backend>::Error> {
        let cf_output_id_to_output = self.inner.cf_handle(CF_OUTPUT_ID_TO_OUTPUT).unwrap();

        // Packing to bytes can't fail.
        self.inner.put_cf(
            &cf_output_id_to_output,
            output_id.pack_new().unwrap(),
            output.pack_new().unwrap(),
        )?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<OutputId, Spent> for Storage {
    async fn insert(&self, output_id: &OutputId, spent: &Spent) -> Result<(), <Self as Backend>::Error> {
        let cf_output_id_to_spent = self.inner.cf_handle(CF_OUTPUT_ID_TO_SPENT).unwrap();

        // Packing to bytes can't fail.
        self.inner.put_cf(
            &cf_output_id_to_spent,
            output_id.pack_new().unwrap(),
            spent.pack_new().unwrap(),
        )?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<Unspent, ()> for Storage {
    async fn insert(&self, unspent: &Unspent, (): &()) -> Result<(), <Self as Backend>::Error> {
        let cf_output_id_unspent = self.inner.cf_handle(CF_OUTPUT_ID_UNSPENT).unwrap();

        // Packing to bytes can't fail.
        self.inner
            .put_cf(&cf_output_id_unspent, unspent.pack_new().unwrap(), [])?;

        Ok(())
    }
}
