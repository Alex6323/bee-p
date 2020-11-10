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
use bee_message::{Message, MessageId};
use bee_storage::access::Iter;

use rocksdb::{DBIterator, IteratorMode};

use std::{marker::PhantomData, ops::Deref};

pub struct StorageIterator<'a, K, V> {
    inner: DBIterator<'a>,
    marker: PhantomData<(K, V)>,
}

impl<'a, K, V> From<DBIterator<'a>> for StorageIterator<'a, K, V> {
    fn from(inner: DBIterator<'a>) -> Self {
        StorageIterator::<K, V> {
            inner,
            marker: PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<'a> Iter<'a, MessageId, Message> for Storage {
    type Iterator = StorageIterator<'a, MessageId, Message>;

    async fn iter(&'a self) -> Result<Self::Iterator, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf_message_id_to_message = self.inner.cf_handle(CF_MESSAGE_ID_TO_MESSAGE).unwrap();

        Ok(self
            .inner
            .iterator_cf(cf_message_id_to_message, IteratorMode::Start)
            .into())
    }
}

impl<'a> Iterator for StorageIterator<'a, MessageId, Message> {
    type Item = (MessageId, Message);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(message_id, message)| {
            (
                MessageId::unpack(&mut message_id.deref()).unwrap(),
                Message::unpack(&mut message.deref()).unwrap(),
            )
        })
    }
}
