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
use super::OpError;
#[cfg(feature = "rocks_db")]
use crate::storage::rocksdb::{WriteBatch, WriteOptions, MILESTONE_HASH_TO_INDEX};
use crate::{persistable::Persistable, storage::Storage};
use std::collections::HashMap;

#[async_trait::async_trait]
#[cfg(feature = "rocks_db")]
pub trait MilestoneIndexOps<H: Persistable + Sync> {
    async fn insert(&self, hash: &H, storage: &Storage) -> Result<(), OpError>
    where
        Self: Persistable + Sized,
    {
        // get column family handle to ms_hash_to_ms_index table in order presist the ms_index;
        let ms_hash_to_ms_index = storage.inner.cf_handle(MILESTONE_HASH_TO_INDEX).unwrap();
        let mut hash_buf = Vec::new();
        hash.encode_persistable(&mut hash_buf);
        let mut index_buf = Vec::new();
        self.encode_persistable(&mut index_buf);
        storage
            .inner
            .put_cf(&ms_hash_to_ms_index, hash_buf.as_slice(), index_buf.as_slice())?;
        Ok(())
    }
    async fn insert_batch(milestones: &HashMap<H, Self>, storage: &Storage) -> Result<(), OpError>
    where
        Self: Persistable + Sized + Sync,
    {
        let mut batch = WriteBatch::default();
        let ms_hash_to_ms_index = storage.inner.cf_handle(MILESTONE_HASH_TO_INDEX).unwrap();
        // reusable buffers
        let mut hash_buf: Vec<u8> = Vec::new();
        let mut index_buf: Vec<u8> = Vec::new();
        for (hash, index) in milestones {
            hash.encode_persistable(&mut hash_buf);
            index.encode_persistable(&mut index_buf);
            batch.put_cf(&ms_hash_to_ms_index, hash_buf.as_slice(), index_buf.as_slice());
            // note: for optimization reason we used buf.set_len = 0 instead of clear()
            unsafe { hash_buf.set_len(0) };
            unsafe { index_buf.set_len(0) };
        }
        let mut write_options = WriteOptions::default();
        write_options.set_sync(false);
        write_options.disable_wal(true);
        storage.inner.write_opt(batch, &write_options)?;
        Ok(())
    }
    async fn remove(hash: &H, storage: &Storage) -> Result<(), OpError> {
        let db = &storage.inner;
        let ms_hash_to_ms_index = db.cf_handle(MILESTONE_HASH_TO_INDEX).unwrap();
        let mut hash_buf = Vec::new();
        hash.encode_persistable(&mut hash_buf);
        db.delete_cf(&ms_hash_to_ms_index, hash_buf.as_slice())?;
        Ok(())
    }
    async fn find_by_hash(hash: &H, storage: &Storage) -> Result<Option<Self>, OpError>
    where
        Self: Persistable + Sized,
    {
        let ms_hash_to_ms_index = storage.inner.cf_handle(MILESTONE_HASH_TO_INDEX).unwrap();
        let mut hash_buf: Vec<u8> = Vec::new();
        hash.encode_persistable(&mut hash_buf);
        if let Some(res) = storage.inner.get_cf(&ms_hash_to_ms_index, hash_buf.as_slice())? {
            let ms_index: Self = Self::decode_persistable(res.as_slice());
            Ok(Some(ms_index))
        } else {
            Ok(None)
        }
    }
}
