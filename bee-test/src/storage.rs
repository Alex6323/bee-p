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

use bee_storage_rocksdb::config::{RocksDBConfig, RocksDBConfigBuilder};

fn get_config() -> RocksDBConfig {
    let buf = std::fs::read_to_string("../bee-storage/bee-storage-rocksdb/config.toml").unwrap();
    toml::from_str::<RocksDBConfigBuilder>(&buf)
        .expect("Failed to deserialize config data")
        .into()
}

#[allow(dead_code)]
async fn start_and_shutdown_rocksdb_storage() {
    // import storage
    use bee_storage_rocksdb::storage::{Backend, Storage};
    // start storage
    let storage: Storage = Storage::start(get_config()).await.unwrap();
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
    let storage: Storage = Storage::start(get_config()).await.unwrap();
    // create empty ledger_diff
    let ledger_diff: LedgerDiff = LedgerDiff::new();
    // milestone_index
    let ms = MilestoneIndex(0);
    // persist it
    assert!(storage.insert(&ms, &ledger_diff).await.is_ok());
    // fetch it
    let result = Fetch::<MilestoneIndex, LedgerDiff>::fetch(&storage, &ms).await;
    // let result = storage.fetch(&ms).await;
    if let Ok(same_ledger_diff) = result {
        assert!(same_ledger_diff.is_some());
    } else {
        panic!("persist_ledger_diff test")
    };
    // delete
    assert!(Delete::<MilestoneIndex, LedgerDiff>::delete(&storage, &ms)
        .await
        .is_ok());
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
    let storage: Storage = Storage::start(get_config()).await.unwrap();
    // milestone_index
    let ms = MilestoneIndex(0);
    // create empty ledger_diff
    let ledger_diff: LedgerDiff = LedgerDiff::new();
    // create batch and insert ledgerDiff
    let mut batch = storage.create_batch().insert(&ms, &ledger_diff);
    // later on delete or insert something
    batch = BatchBuilder::<'_, Storage, MilestoneIndex, LedgerDiff>::delete(batch, &ms);
    batch.apply(true).await.unwrap();
    let result: Result<Option<LedgerDiff>, _> = storage.fetch(&ms).await;
    assert!(result.unwrap().is_none());
    // shutdown storage
    assert!(storage.shutdown().await.is_ok())
}

#[tokio::test]
async fn storage() {
    start_and_shutdown_rocksdb_storage().await;
    persist_ledger_diff().await;
    batch_storage().await;
}
