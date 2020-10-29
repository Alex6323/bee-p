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
use bee_storage::access::Insert;

use blake2::Blake2b;

#[async_trait::async_trait]
impl Insert<MessageId, Message> for Storage {
    type Error = OpError;

    async fn insert(&self, message_id: &MessageId, message: &Message) -> Result<(), Self::Error> {
        let message_id_to_message = self.inner.cf_handle(MESSAGE_ID_TO_MESSAGE).unwrap();

        let mut message_buf = Vec::with_capacity(message.packed_len());
        // Packing to bytes can't fail.
        message.pack(&mut message_buf).unwrap();

        self.inner.put_cf(&message_id_to_message, message_id, message_buf)?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<TransactionId, Transaction> for Storage {
    type Error = OpError;

    async fn insert(&self, transaction_id: &TransactionId, transaction: &Transaction) -> Result<(), Self::Error> {
        let transaction_id_to_transaction = self.inner.cf_handle(TRANSACTION_ID_TO_TRANSACTION).unwrap();

        let mut transaction_buf = Vec::with_capacity(transaction.packed_len());
        // Packing to bytes can't fail.
        transaction.pack(&mut transaction_buf).unwrap();

        self.inner
            .put_cf(&transaction_id_to_transaction, transaction_id, transaction_buf)?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<(HashedIndex<Blake2b>, MessageId), ()> for Storage {
    type Error = OpError;

    async fn insert(
        &self,
        (index, message_id): &(HashedIndex<Blake2b>, MessageId),
        (): &(),
    ) -> Result<(), Self::Error> {
        let payload_index_to_message_id = self.inner.cf_handle(PAYLOAD_INDEX_TO_MESSAGE_ID).unwrap();

        let mut key = index.as_ref().to_vec();
        key.extend_from_slice(message_id.as_ref());

        self.inner.put_cf(&payload_index_to_message_id, key, [])?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<(HashedIndex<Blake2b>, TransactionId), ()> for Storage {
    type Error = OpError;

    async fn insert(
        &self,
        (index, transaction_id): &(HashedIndex<Blake2b>, TransactionId),
        (): &(),
    ) -> Result<(), Self::Error> {
        let payload_index_to_transaction_id = self.inner.cf_handle(PAYLOAD_INDEX_TO_TRANSACTION_ID).unwrap();

        let mut key = index.as_ref().to_vec();
        key.extend_from_slice(transaction_id.as_ref());

        self.inner.put_cf(&payload_index_to_transaction_id, key, [])?;

        Ok(())
    }
}
