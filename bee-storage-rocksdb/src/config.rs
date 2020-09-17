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

use crate::{compaction::CompactionStyle, compression::CompressionType};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub rocksdb: RocksDB,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RocksDB {
    pub path: String,
    pub create_if_missing: Option<bool>,
    pub create_missing_column_families: Option<bool>,
    pub enable_statistics: Option<bool>,
    pub increase_parallelism: Option<i32>,
    pub optimize_for_point_lookup: Option<u64>,
    pub optimize_level_style_compaction: Option<usize>,
    pub optimize_universal_style_compaction: Option<usize>,
    pub set_advise_random_on_open: Option<bool>,
    pub set_allow_concurrent_memtable_write: Option<bool>,
    pub set_allow_mmap_reads: Option<bool>,
    pub set_allow_mmap_writes: Option<bool>,
    pub set_atomic_flush: Option<bool>,
    pub set_bytes_per_sync: Option<u64>,
    pub set_compaction_readahead_size: Option<usize>,
    pub set_compaction_style: Option<CompactionStyle>,
    pub set_max_write_buffer_number: Option<i32>,
    pub set_max_background_compactions: Option<i32>,
    pub set_max_background_flushes: Option<i32>,
    pub set_disable_auto_compactions: Option<bool>,
    pub set_compression_type: Option<CompressionType>,
}
