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
