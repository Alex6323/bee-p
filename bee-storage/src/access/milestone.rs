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
use crate::{persistable::Persistable, storage::Backend};
use std::collections::HashMap;

#[async_trait::async_trait]
pub trait MilestoneOps<H, S, E> {
    async fn insert(&self, storage: &S) -> Result<(), E>
    where
        Self: Sized,
        S: Backend;
    async fn insert_batch(transactions: &HashMap<H, Self>, storage: &S) -> Result<(), E>
    where
        Self: Sized,
        H: Persistable,
        S: Backend;
    async fn remove(hash: &H, storage: &S) -> Result<(), E>
    where
        Self: Sized,
        H: Persistable,
        S: Backend;
    async fn find_by_hash(hash: &H, storage: &S) -> Result<Option<Self>, E>
    where
        Self: Sized,
        H: Persistable,
        S: Backend;
}

#[macro_export]
#[cfg(feature = "rocks_db")]
macro_rules! impl_milestone_ops {
    ($object:ty) => {
        use bee_storage::{
            access::{MilestoneOps, OpError},
            storage::{rocksdb::*, Backend, Storage},
        };
        use std::collections::HashMap;
        #[async_trait::async_trait]
        impl MilestoneOps<Hash, Storage, OpError> for $object {
            async fn insert(&self, storage: &Storage) -> Result<(), OpError> {
                // get column family handle to ms_hash_to_ms_index table in order presist the ms_index;
                let ms_hash_to_ms_index = storage.inner.cf_handle(MILESTONE_HASH_TO_INDEX).unwrap();
                let mut hash_buf = Vec::new();
                self.hash().encode_persistable(&mut hash_buf);
                let mut index_buf = Vec::new();
                self.index().encode_persistable(&mut index_buf);
                storage
                    .inner
                    .put_cf(&ms_hash_to_ms_index, hash_buf.as_slice(), index_buf.as_slice())?;
                Ok(())
            }
            async fn insert_batch(transactions: &HashMap<Hash, Self>, storage: &Storage) -> Result<(), OpError> {
                let mut batch = WriteBatch::default();
                let ms_hash_to_ms_index = storage.inner.cf_handle(MILESTONE_HASH_TO_INDEX).unwrap();
                // reusable buffers
                let mut hash_buf: Vec<u8> = Vec::new();
                let mut index_buf: Vec<u8> = Vec::new();
                for (hash, ms_index) in transactions {
                    hash.encode_persistable(&mut hash_buf);
                    ms_index.encode_persistable(&mut index_buf);
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
            async fn remove(hash: &Hash, storage: &Storage) -> Result<(), OpError> {
                let db = &storage.inner;
                let ms_hash_to_ms_index = db.cf_handle(MILESTONE_HASH_TO_INDEX).unwrap();
                let mut hash_buf = Vec::new();
                hash.encode_persistable(&mut hash_buf);
                db.delete_cf(&ms_hash_to_ms_index, hash_buf.as_slice())?;
                Ok(())
            }
            async fn find_by_hash(hash: &Hash, storage: &Storage) -> Result<Option<Self>, OpError> {
                let ms_hash_to_ms_index = storage.inner.cf_handle(MILESTONE_HASH_TO_INDEX).unwrap();
                let mut hash_buf: Vec<u8> = Vec::new();
                hash.encode_persistable(&mut hash_buf);
                if let Some(res) = storage.inner.get_cf(&ms_hash_to_ms_index, hash_buf.as_slice())? {
                    let ms_index: MilestoneIndex = MilestoneIndex::decode_persistable(res.as_slice(), res.len());
                    Ok(Some(Milestone::new(hash, ms_index)))
                } else {
                    Ok(None)
                }
            }
        }
    };
}
