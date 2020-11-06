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
use bee_storage::access::Exist;

#[async_trait::async_trait]
impl Exist<MessageId, Message> for Storage {
    async fn exist(&self, message_id: &MessageId) -> Result<bool, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf_message_id_to_message = self.inner.cf_handle(CF_MESSAGE_ID_TO_MESSAGE).unwrap();

        Ok(self.inner.get_cf(&cf_message_id_to_message, message_id)?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<MessageId, Vec<MessageId>> for Storage {
    async fn exist(&self, parent: &MessageId) -> Result<bool, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf_message_id_to_message_id = self.inner.cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID).unwrap();

        let mut iterator = self.inner.prefix_iterator_cf(&cf_message_id_to_message_id, parent);
        let exist = iterator.next().is_some();

        match iterator.status() {
            Ok(_) => Ok(exist),
            Err(e) => Err(e)?,
        }
    }
}

#[async_trait::async_trait]
impl Exist<HashedIndex, Vec<MessageId>> for Storage {
    async fn exist(&self, index: &HashedIndex) -> Result<bool, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf_index_to_message_id = self.inner.cf_handle(CF_INDEX_TO_MESSAGE_ID).unwrap();

        let mut iterator = self.inner.prefix_iterator_cf(&cf_index_to_message_id, index);
        let exist = iterator.next().is_some();

        match iterator.status() {
            Ok(_) => Ok(exist),
            Err(e) => Err(e)?,
        }
    }
}

#[async_trait::async_trait]
impl Exist<OutputId, Output> for Storage {
    async fn exist(&self, output_id: &OutputId) -> Result<bool, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf_output_id_to_output = self.inner.cf_handle(CF_OUTPUT_ID_TO_OUTPUT).unwrap();

        // Packing to bytes can't fail.
        Ok(self
            .inner
            .get_cf(&cf_output_id_to_output, output_id.pack_new().unwrap())?
            .is_some())
    }
}

#[async_trait::async_trait]
impl Exist<OutputId, Spent> for Storage {
    async fn exist(&self, output_id: &OutputId) -> Result<bool, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf_output_id_to_spent = self.inner.cf_handle(CF_OUTPUT_ID_TO_SPENT).unwrap();

        // Packing to bytes can't fail.
        Ok(self
            .inner
            .get_cf(&cf_output_id_to_spent, output_id.pack_new().unwrap())?
            .is_some())
    }
}

#[async_trait::async_trait]
impl Exist<Unspent, ()> for Storage {
    async fn exist(&self, unspent: &Unspent) -> Result<bool, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf_output_id_unspent = self.inner.cf_handle(CF_OUTPUT_ID_UNSPENT).unwrap();

        // Packing to bytes can't fail.
        Ok(self
            .inner
            .get_cf(&cf_output_id_unspent, unspent.pack_new().unwrap())?
            .is_some())
    }
}
