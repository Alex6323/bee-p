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
use bee_ledger::{output::Output, spent::Spent};
use bee_message::{
    payload::{
        indexation::{HashedIndex, HASHED_INDEX_SIZE},
        transaction::OutputId,
    },
    Message, MessageId, MESSAGE_ID_LENGTH,
};
use bee_protocol::tangle::MessageMetadata;
use bee_storage::access::Fetch;

use std::convert::TryInto;

#[async_trait::async_trait]
impl Fetch<MessageId, Message> for Storage {
    async fn fetch(&self, message_id: &MessageId) -> Result<Option<Message>, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf_message_id_to_message = self.inner.cf_handle(CF_MESSAGE_ID_TO_MESSAGE).unwrap();

        if let Some(res) = self.inner.get_cf(&cf_message_id_to_message, message_id)? {
            Ok(Some(Message::unpack(&mut res.as_slice()).unwrap()))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl Fetch<MessageId, MessageMetadata> for Storage {
    async fn fetch(&self, message_id: &MessageId) -> Result<Option<MessageMetadata>, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf_message_id_to_metadata = self.inner.cf_handle(CF_MESSAGE_ID_TO_METADATA).unwrap();

        if let Some(res) = self.inner.get_cf(&cf_message_id_to_metadata, message_id)? {
            Ok(Some(MessageMetadata::unpack(&mut res.as_slice()).unwrap()))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl Fetch<MessageId, Vec<MessageId>> for Storage {
    async fn fetch(&self, parent: &MessageId) -> Result<Option<Vec<MessageId>>, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf_message_id_to_message_id = self.inner.cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID).unwrap();

        Ok(Some(
            self.inner
                .prefix_iterator_cf(&cf_message_id_to_message_id, parent)
                .map(|(key, _)| {
                    let (_, child) = key.split_at(HASHED_INDEX_SIZE);
                    let child: [u8; MESSAGE_ID_LENGTH] = child.try_into().unwrap();
                    MessageId::from(child)
                })
                .take(self.config.fetch_edge_limit)
                .collect(),
        ))
    }
}

#[async_trait::async_trait]
impl Fetch<HashedIndex, Vec<MessageId>> for Storage {
    async fn fetch(&self, index: &HashedIndex) -> Result<Option<Vec<MessageId>>, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf_index_to_message_id = self.inner.cf_handle(CF_INDEX_TO_MESSAGE_ID).unwrap();

        Ok(Some(
            self.inner
                .prefix_iterator_cf(&cf_index_to_message_id, index)
                .map(|(key, _)| {
                    let (_, message_id) = key.split_at(HASHED_INDEX_SIZE);
                    let message_id: [u8; MESSAGE_ID_LENGTH] = message_id.try_into().unwrap();
                    MessageId::from(message_id)
                })
                .take(self.config.fetch_index_limit)
                .collect(),
        ))
    }
}

#[async_trait::async_trait]
impl Fetch<OutputId, Output> for Storage {
    async fn fetch(&self, output_id: &OutputId) -> Result<Option<Output>, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf_output_id_to_output = self.inner.cf_handle(CF_OUTPUT_ID_TO_OUTPUT).unwrap();

        if let Some(res) = self.inner.get_cf(&cf_output_id_to_output, output_id.pack_new())? {
            Ok(Some(Output::unpack(&mut res.as_slice()).unwrap()))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl Fetch<OutputId, Spent> for Storage {
    async fn fetch(&self, output_id: &OutputId) -> Result<Option<Spent>, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf_output_id_to_spent = self.inner.cf_handle(CF_OUTPUT_ID_TO_SPENT).unwrap();

        if let Some(res) = self.inner.get_cf(&cf_output_id_to_spent, output_id.pack_new())? {
            Ok(Some(Spent::unpack(&mut res.as_slice()).unwrap()))
        } else {
            Ok(None)
        }
    }
}
