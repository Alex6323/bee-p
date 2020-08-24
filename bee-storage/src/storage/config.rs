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

//! A crate that contains foundational building blocks for the IOTA Tangle.

use rocksdb::{DBCompactionStyle, DBCompressionType};
use serde::Deserialize;
use std::convert::From;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[cfg(feature = "rocks_db")]
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

#[cfg(feature = "rocks_db")]
#[derive(Debug, Clone, Deserialize)]
pub enum CompactionStyle {
    Level,
    Universal,
    Fifo,
}
#[cfg(feature = "rocks_db")]
impl From<CompactionStyle> for DBCompactionStyle {
    fn from(compaction_style: CompactionStyle) -> Self {
        match compaction_style {
            CompactionStyle::Level => DBCompactionStyle::Level,
            CompactionStyle::Universal => DBCompactionStyle::Universal,
            CompactionStyle::Fifo => DBCompactionStyle::Fifo,
        }
    }
}

#[cfg(feature = "rocks_db")]
#[derive(Debug, Clone, Deserialize)]
pub enum CompressionType {
    None,
    Snappy,
    Zlib,
    Bz2,
    Lz4,
    Lz4hc,
    Zstd,
}
#[cfg(feature = "rocks_db")]
impl From<CompressionType> for DBCompressionType {
    fn from(compression_type: CompressionType) -> Self {
        match compression_type {
            CompressionType::None => DBCompressionType::None,
            CompressionType::Snappy => DBCompressionType::Snappy,
            CompressionType::Zlib => DBCompressionType::Zlib,
            CompressionType::Bz2 => DBCompressionType::Bz2,
            CompressionType::Lz4 => DBCompressionType::Lz4,
            CompressionType::Lz4hc => DBCompressionType::Lz4hc,
            CompressionType::Zstd => DBCompressionType::Zstd,
        }
    }
}
