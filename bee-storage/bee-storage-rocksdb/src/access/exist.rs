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
impl Exist<MessageId, MessageMetadata> for Storage {
    async fn exist(&self, message_id: &MessageId) -> Result<bool, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf_message_id_to_metadata = self.inner.cf_handle(CF_MESSAGE_ID_TO_METADATA).unwrap();

        Ok(self.inner.get_cf(&cf_message_id_to_metadata, message_id)?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<(MessageId, MessageId), ()> for Storage {
    async fn exist(&self, (parent, child): &(MessageId, MessageId)) -> Result<bool, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf_message_id_to_message_id = self.inner.cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID).unwrap();

        let mut key = parent.as_ref().to_vec();
        key.extend_from_slice(child.as_ref());

        Ok(self.inner.get_cf(&cf_message_id_to_message_id, key)?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<(HashedIndex, MessageId), ()> for Storage {
    async fn exist(&self, (index, message_id): &(HashedIndex, MessageId)) -> Result<bool, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf_index_to_message_id = self.inner.cf_handle(CF_INDEX_TO_MESSAGE_ID).unwrap();

        let mut key = index.as_ref().to_vec();
        key.extend_from_slice(message_id.as_ref());

        Ok(self.inner.get_cf(&cf_index_to_message_id, key)?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<OutputId, Output> for Storage {
    async fn exist(&self, output_id: &OutputId) -> Result<bool, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf_output_id_to_output = self.inner.cf_handle(CF_OUTPUT_ID_TO_OUTPUT).unwrap();

        Ok(self
            .inner
            .get_cf(&cf_output_id_to_output, output_id.pack_new())?
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

        Ok(self
            .inner
            .get_cf(&cf_output_id_to_spent, output_id.pack_new())?
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

        Ok(self.inner.get_cf(&cf_output_id_unspent, unspent.pack_new())?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<(Ed25519Address, OutputId), ()> for Storage {
    async fn exist(&self, (address, output_id): &(Ed25519Address, OutputId)) -> Result<bool, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf_ed25519_address_to_output_id = self.inner.cf_handle(CF_ED25519_ADDRESS_TO_OUTPUT_ID).unwrap();

        let mut key = address.as_ref().to_vec();
        key.extend_from_slice(&output_id.pack_new());

        Ok(self.inner.get_cf(&cf_ed25519_address_to_output_id, key)?.is_some())
    }
}
