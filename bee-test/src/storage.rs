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

#[allow(dead_code)]
async fn start_and_shutdown_rocksdb_storage() {
    // import storage
    use bee_storage_rocksdb::storage::{Backend, Storage};
    // start storage
    let storage: Storage = Storage::start("../bee-storage/bee-storage-rocksdb/config.toml".to_string())
        .await
        .unwrap();
    // shutdown storage
    assert!(storage.shutdown().await.is_ok())
}

#[allow(dead_code)]
async fn persist_ledger_diff() {
    // imports
    use bee_ledger::diff::*;
    use bee_protocol::MilestoneIndex;
    use bee_storage::access::{Delete, Fetch, Insert};
    use bee_storage_rocksdb::storage::{Backend, Storage};
    // start storage
    let storage: Storage = Storage::start("../bee-storage/bee-storage-rocksdb/config.toml".to_string())
        .await
        .unwrap();
    // create empty ledger_diff
    let ledger_diff: LedgerDiff = LedgerDiff::new();
    // milestone_index
    let ms = MilestoneIndex(0);
    // persist it
    assert!(storage.insert(&ms, &ledger_diff).await.is_ok());
    // find it
    if let Ok(same_ledger_diff) = storage.fetch(&ms).await {
        assert!(same_ledger_diff.is_some());
    } else {
        panic!("persist_ledger_diff test")
    };
    // delete
    assert!(storage.delete(&ms).await.is_ok());
    // shutdown storage
    assert!(storage.shutdown().await.is_ok())
}

#[allow(dead_code)]
async fn batch_storage() {
    // imports
    use bee_ledger::diff::*;
    use bee_protocol::MilestoneIndex;
    use bee_storage::access::*;
    use bee_storage_rocksdb::storage::{Backend, Storage};
    // start storage
    let storage: Storage = Storage::start("../bee-storage/bee-storage-rocksdb/config.toml".to_string())
        .await
        .unwrap();
    // milestone_index
    let ms = MilestoneIndex(0);
    // create empty ledger_diff
    let ledger_diff: LedgerDiff = LedgerDiff::new();
    // create batch
    let batch = storage.create_batch();
    // later on insert something
    batch.insert(&ms, &ledger_diff).delete(&ms).apply(true).await.unwrap();
    assert!(storage.fetch(&ms).await.unwrap().is_none());
    // shutdown storage
    assert!(storage.shutdown().await.is_ok())
}

#[tokio::test]
async fn storage() {
    start_and_shutdown_rocksdb_storage().await;
    persist_ledger_diff().await;
    batch_storage().await;
}
