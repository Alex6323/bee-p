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

use super::config::Config;
pub use bytemuck::*;
pub use rocksdb::*;
use std::error::Error;

pub const TRANSACTION_HASH_TO_TRANSACTION: &str = "transaction_hash_to_transaction";
pub const TRANSACTION_HASH_TO_METADATA: &str = "transaction_hash_to_metadata";
pub const MILESTONE_HASH_TO_INDEX: &str = "milestone_hash_to_index";
pub const MILESTONE_INDEX_TO_HASH: &str = "milestone_index_to_hash";
pub const MILESTONE_INDEX_TO_DELTA: &str = "milestone_hash_to_delta";

pub struct RocksdbBackend;

impl RocksdbBackend {
    pub fn new(config: Config) -> Result<DB, Box<dyn Error>> {
        let transaction_hash_to_transaction =
            ColumnFamilyDescriptor::new(TRANSACTION_HASH_TO_TRANSACTION, Options::default());
        let transaction_hash_to_transaction_metadata =
            ColumnFamilyDescriptor::new(TRANSACTION_HASH_TO_METADATA, Options::default());
        let milestone_hash_to_index = ColumnFamilyDescriptor::new(MILESTONE_HASH_TO_INDEX, Options::default());
        let milestone_index_to_delta = ColumnFamilyDescriptor::new(MILESTONE_INDEX_TO_DELTA, Options::default());
        let mut opts = Options::default();
        if let Some(create_if_missing) = config.create_if_missing {
            opts.create_if_missing(create_if_missing);
        }
        if let Some(create_missing_column_families) = config.create_missing_column_families {
            opts.create_missing_column_families(create_missing_column_families);
        }
        if let Some(enable_statistics) = config.enable_statistics {
            if enable_statistics {
                opts.enable_statistics();
            }
        }
        if let Some(increase_parallelism) = config.increase_parallelism {
            opts.increase_parallelism(increase_parallelism);
        }
        if let Some(optimize_for_point_lookup) = config.optimize_for_point_lookup {
            opts.optimize_for_point_lookup(optimize_for_point_lookup);
        }
        if let Some(optimize_level_style_compaction) = config.optimize_level_style_compaction {
            opts.optimize_level_style_compaction(optimize_level_style_compaction);
        }
        if let Some(optimize_universal_style_compaction) = config.optimize_universal_style_compaction {
            opts.optimize_universal_style_compaction(optimize_universal_style_compaction);
        }
        if let Some(set_advise_random_on_open) = config.set_advise_random_on_open {
            opts.set_advise_random_on_open(set_advise_random_on_open);
        }
        if let Some(set_allow_concurrent_memtable_write) = config.set_allow_concurrent_memtable_write {
            opts.set_allow_concurrent_memtable_write(set_allow_concurrent_memtable_write);
        }
        if let Some(set_allow_mmap_reads) = config.set_allow_mmap_reads {
            opts.set_allow_mmap_reads(set_allow_mmap_reads);
        }
        if let Some(set_allow_mmap_writes) = config.set_allow_mmap_writes {
            opts.set_allow_mmap_writes(set_allow_mmap_writes);
        }
        if let Some(set_atomic_flush) = config.set_atomic_flush {
            opts.set_atomic_flush(set_atomic_flush);
        }
        if let Some(set_bytes_per_sync) = config.set_bytes_per_sync {
            opts.set_bytes_per_sync(set_bytes_per_sync);
        }
        if let Some(set_compaction_readahead_size) = config.set_compaction_readahead_size {
            opts.set_compaction_readahead_size(set_compaction_readahead_size);
        }
        if let Some(set_compaction_style) = config.set_compaction_style {
            opts.set_compaction_style(DBCompactionStyle::from(set_compaction_style));
        }
        if let Some(set_max_write_buffer_number) = config.set_max_write_buffer_number {
            opts.set_max_write_buffer_number(set_max_write_buffer_number);
        }
        if let Some(set_max_background_compactions) = config.set_max_background_compactions {
            opts.set_max_background_compactions(set_max_background_compactions);
        }
        if let Some(set_max_background_flushes) = config.set_max_background_flushes {
            opts.set_max_background_flushes(set_max_background_flushes);
        }
        if let Some(set_disable_auto_compactions) = config.set_disable_auto_compactions {
            opts.set_disable_auto_compactions(set_disable_auto_compactions);
        }
        if let Some(set_compression_type) = config.set_compression_type {
            opts.set_compression_type(DBCompressionType::from(set_compression_type));
        }
        let column_familes = vec![
            transaction_hash_to_transaction,
            transaction_hash_to_transaction_metadata,
            milestone_hash_to_index,
            milestone_index_to_delta,
        ];
        let db = DB::open_cf_descriptors(&opts, config.path, column_familes)?;
        Ok(db)
    }
}
