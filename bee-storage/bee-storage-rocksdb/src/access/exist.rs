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
use bee_storage::access::Exist;

use blake2::Blake2b;

#[async_trait::async_trait]
impl Exist<MessageId, Message> for Storage {
    type Error = OpError;

    async fn exist(&self, message_id: &MessageId) -> Result<bool, Self::Error>
    where
        Self: Sized,
    {
        let cf_message_id_to_message = self.inner.cf_handle(CF_MESSAGE_ID_TO_MESSAGE).unwrap();

        Ok(self.inner.get_cf(&cf_message_id_to_message, message_id)?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<MessageId, Vec<MessageId>> for Storage {
    type Error = OpError;

    async fn exist(&self, parent: &MessageId) -> Result<bool, Self::Error>
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
impl Exist<HashedIndex<Blake2b>, Vec<MessageId>> for Storage {
    type Error = OpError;

    async fn exist(&self, index: &HashedIndex<Blake2b>) -> Result<bool, Self::Error>
    where
        Self: Sized,
    {
        let cf_payload_index_to_message_id = self.inner.cf_handle(CF_PAYLOAD_INDEX_TO_MESSAGE_ID).unwrap();

        let mut iterator = self.inner.prefix_iterator_cf(&cf_payload_index_to_message_id, index);
        let exist = iterator.next().is_some();

        match iterator.status() {
            Ok(_) => Ok(exist),
            Err(e) => Err(e)?,
        }
    }
}

#[async_trait::async_trait]
impl Exist<OutputId, Spent> for Storage {
    type Error = OpError;

    async fn exist(&self, output_id: &OutputId) -> Result<bool, Self::Error>
    where
        Self: Sized,
    {
        let cf_output_id_to_spent = self.inner.cf_handle(CF_OUTPUT_ID_TO_SPENT).unwrap();

        let mut output_id_buf = Vec::with_capacity(output_id.packed_len());
        // Packing to bytes can't fail.
        output_id.pack(&mut output_id_buf).unwrap();

        Ok(self.inner.get_cf(&cf_output_id_to_spent, output_id_buf)?.is_some())
    }
}
