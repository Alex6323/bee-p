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

use bee_message::{payload::indexation::HashedIndex, Message, MessageId};
use bee_storage::access::Delete;

use blake2::Blake2b;

#[async_trait::async_trait]
impl Delete<MessageId, Message> for Storage {
    type Error = OpError;

    async fn delete(&self, message_id: &MessageId) -> Result<(), Self::Error> {
        let message_id_to_message = self.inner.cf_handle(MESSAGE_ID_TO_MESSAGE).unwrap();

        self.inner.delete_cf(&message_id_to_message, message_id)?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<(HashedIndex<Blake2b>, MessageId), ()> for Storage {
    type Error = OpError;

    async fn delete(&self, (index, message_id): &(HashedIndex<Blake2b>, MessageId)) -> Result<(), Self::Error> {
        let payload_index_to_message_id = self.inner.cf_handle(PAYLOAD_INDEX_TO_MESSAGE_ID).unwrap();

        let mut key = index.as_ref().to_vec();
        key.extend_from_slice(message_id.as_ref());

        self.inner.delete_cf(&payload_index_to_message_id, key)?;

        Ok(())
    }
}
