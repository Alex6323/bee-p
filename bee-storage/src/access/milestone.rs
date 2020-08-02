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
#[macro_export]
macro_rules! impl_milestone_ops {
    ($object:ty) => {
        use bee_storage::access::OpError;
        use bee_transaction::bundled::BundledTransactionField;
        use bee_storage::storage::rocksdb::*;
        use bee_storage::storage::Storage;
        use bee_ternary::{T5B1Buf, TritBuf, T5B1};
        use std::collections::HashSet;
        #[cfg(feature = "rocks_db")]
        impl $object {
            async fn insert(&self, storage: &Storage) -> Result<(), OpError> {
                let db = &storage.inner;
                let milestone_cf_hash_to_index = db.cf_handle(MILESTONE_CF_HASH_TO_INDEX).unwrap();
                let milestone_cf_index_to_hash = db.cf_handle(MILESTONE_CF_INDEX_TO_HASH).unwrap();
                let hash_buf = self.hash().to_inner().encode::<T5B1Buf>();
                db.put_cf(
                    &milestone_cf_hash_to_index,
                    cast_slice(hash_buf.as_i8_slice()),
                    self.index().to_le_bytes(),
                )?;
                db.put_cf(
                    &milestone_cf_index_to_hash,
                    self.index().to_le_bytes(),
                    cast_slice(hash_buf.as_i8_slice()),
                )?;
                Ok(())
            }
            async fn set_milestone_index_batch(&self, hashes: &HashSet<Hash>, storage: &Storage) -> Result<(), OpError> {
                let mut batch = WriteBatch::default();
                let db = &storage.inner;
                let hash_to_snapshot_index = db.cf_handle(TRANSACTION_CF_HASH_TO_SNAPSHOT_INDEX).unwrap();
                for hash in hashes {
                    let hash_buf = hash.to_inner().encode::<T5B1Buf>();
                    batch.put_cf(
                        &hash_to_snapshot_index,
                        cast_slice(hash_buf.as_i8_slice()),
                        self.index.to_le_bytes(),
                    );
                }
                let mut write_options = WriteOptions::default();
                write_options.set_sync(false);
                write_options.disable_wal(true);
                db.write_opt(batch, &write_options)?;
                Ok(())
            }
            async fn find_by_hash(hash: Hash, storage: &Storage) -> Result<Option<Milestone>, OpError> {
                let milestone_cf_hash_to_index = storage.inner.cf_handle(MILESTONE_CF_HASH_TO_INDEX).unwrap();
                if let Some(res) = storage.inner.get_cf(
                    &milestone_cf_hash_to_index,
                    cast_slice(hash.to_inner().encode::<T5B1Buf>().as_i8_slice()),
                )?
                {
                    let mut index_buf: [u8; 4] = [0; 4];
                    index_buf.copy_from_slice(res.as_slice());
                    Ok(Some(Milestone::new(
                        hash,
                        MilestoneIndex(u32::from_le_bytes(index_buf)),
                    )))
                } else {
                    Ok(None)
                }
            }
        }
    };
}
