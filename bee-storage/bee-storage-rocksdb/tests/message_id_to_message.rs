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

use bee_common::packable::Packable;
use bee_message::{Message, MessageId};
use bee_storage::{
    access::{Batch, BatchBuilder, Delete, Exist, Fetch, Insert},
    storage::Backend,
};
use bee_storage_rocksdb::{config::RocksDBConfigBuilder, storage::Storage};
use bee_test::message::{random_message, random_message_id};

#[tokio::test]
async fn access() {
    let config = RocksDBConfigBuilder::default().finish();
    let storage = Storage::start(config).await.unwrap();

    let message_id = random_message_id();
    let message_1 = random_message();

    assert!(!storage.exist(&message_id).await.unwrap());
    assert!(Fetch::<MessageId, Message>::fetch(&storage, &message_id)
        .await
        .unwrap()
        .is_none());

    storage.insert(&message_id, &message_1).await.unwrap();

    assert!(storage.exist(&message_id).await.unwrap());

    let message_2 = Fetch::<MessageId, Message>::fetch(&storage, &message_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(message_1.pack_new().unwrap(), message_2.pack_new().unwrap());

    Delete::<MessageId, Message>::delete(&storage, &message_id)
        .await
        .unwrap();

    assert!(!storage.exist(&message_id).await.unwrap());
    assert!(Fetch::<MessageId, Message>::fetch(&storage, &message_id)
        .await
        .unwrap()
        .is_none());

    let mut message_ids = Vec::new();

    for _ in 0usize..100usize {
        let (message_id, message) = (random_message_id(), random_message());
        message_ids.push(message_id);
        storage.insert(&message_id, &message).await.unwrap();
    }

    let mut batch = Storage::batch_begin();

    for (i, message_id) in message_ids.iter().enumerate() {
        storage
            .batch_insert(&mut batch, &random_message_id(), &random_message())
            .unwrap();
        if i % 2 == 0 {
            storage.batch_delete(&mut batch, message_id).unwrap();
        }
    }

    storage.batch_commit(batch, true).await.unwrap();

    for (i, message_id) in message_ids.iter().enumerate() {
        if i % 2 == 0 {
            assert!(!storage.exist(message_id).await.unwrap());
        } else {
            assert!(storage.exist(message_id).await.unwrap());
        }
    }
}
