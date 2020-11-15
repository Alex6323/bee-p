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
use bee_storage::access::AsStream;

use futures::{
    stream::Stream,
    task::{Context, Poll},
};
use pin_project::pin_project;
use rocksdb::{DBIterator, IteratorMode};

use std::{marker::PhantomData, ops::Deref, pin::Pin};

#[pin_project(project = StorageStreamProj)]
pub struct StorageStream<'a, K, V> {
    #[pin]
    inner: DBIterator<'a>,
    budget: usize,
    counter: usize,
    marker: PhantomData<(K, V)>,
}

impl<'a, K, V> StorageStream<'a, K, V> {
    fn new(inner: DBIterator<'a>, budget: usize) -> Self {
        StorageStream::<K, V> {
            inner,
            budget,
            counter: 0,
            marker: PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<'a> AsStream<'a, MessageId, Message> for Storage {
    type Stream = StorageStream<'a, MessageId, Message>;

    async fn stream(&'a self) -> Result<Self::Stream, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf_message_id_to_message = self.inner.cf_handle(CF_MESSAGE_ID_TO_MESSAGE).unwrap();

        Ok(StorageStream::new(
            self.inner.iterator_cf(cf_message_id_to_message, IteratorMode::Start),
            self.config.iteration_budget,
        ))
    }
}

impl<'a> Stream for StorageStream<'a, MessageId, Message> {
    type Item = (MessageId, Message);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let StorageStreamProj {
            mut inner,
            budget,
            counter,
            ..
        } = self.project();

        if counter == budget {
            *counter = 0;
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }

        *counter += 1;

        let item = inner.next().map(|(message_id, message)| {
            (
                MessageId::unpack(&mut message_id.deref()).unwrap(),
                Message::unpack(&mut message.deref()).unwrap(),
            )
        });

        if inner.valid() {
            Poll::Ready(item)
        } else {
            Poll::Ready(None)
        }
    }
}
