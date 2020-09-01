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
pub trait LedgerDiffOps<K, S, E> {
    async fn insert(&self, milestone_index: &K, storage: &S) -> Result<(), E>
    where
        Self: Persistable + Sized,
        S: Backend;
    async fn insert_batch(ledger_diffs: &HashMap<K, Self>, storage: &S) -> Result<(), E>
    where
        Self: Persistable + Sized,
        K: Persistable,
        S: Backend;
    async fn remove(hash: &K, storage: &S) -> Result<(), E>
    where
        Self: Persistable + Sized,
        K: Persistable,
        S: Backend;
    async fn find_by_milestone_index(milestone_index: &K, storage: &S) -> Result<Option<Self>, E>
    where
        Self: Persistable + Sized,
        K: Persistable,
        S: Backend;
}

#[macro_export]
#[cfg(feature = "rocks_db")]
macro_rules! impl_ledger_diff_ops {
    ($object:ty) => {
        use bee_protocol::MilestoneIndex;
        pub use bee_storage::{
            access::{LedgerDiffOps, OpError},
            storage::{rocksdb::*, Backend, Storage},
        };
        #[async_trait::async_trait]
        impl LedgerDiffOps<MilestoneIndex, Storage, OpError> for $object {
            async fn insert(&self, milestone_index: &MilestoneIndex, storage: &Storage) -> Result<(), OpError> {
                let ms_index_to_ledger_diff = storage.inner.cf_handle(MILESTONE_INDEX_TO_LEDGER_DIFF).unwrap();
                let mut index_buf = Vec::new();
                milestone_index.encode_persistable(&mut index_buf);
                let mut ledger_diff_buf = Vec::new();
                self.encode_persistable(&mut ledger_diff_buf);
                storage.inner.put_cf(
                    &ms_index_to_ledger_diff,
                    index_buf.as_slice(),
                    ledger_diff_buf.as_slice(),
                )?;
                Ok(())
            }
            async fn insert_batch(
                ledger_diffs: &HashMap<MilestoneIndex, Self>,
                storage: &Storage,
            ) -> Result<(), OpError> {
                let mut batch = WriteBatch::default();
                let ms_index_to_ledger_diff = storage.inner.cf_handle(MILESTONE_INDEX_TO_LEDGER_DIFF).unwrap();
                // reusable buffers
                let mut index_buf: Vec<u8> = Vec::new();
                let mut ledger_diff_buf: Vec<u8> = Vec::new();
                for (ms_index, ledger_diff) in ledger_diffs {
                    ms_index.encode_persistable(&mut index_buf);
                    ledger_diff.encode_persistable(&mut ledger_diff_buf);
                    batch.put_cf(
                        &ms_index_to_ledger_diff,
                        index_buf.as_slice(),
                        ledger_diff_buf.as_slice(),
                    );
                    // note: for optimization reason we used buf.set_len = 0 instead of clear()
                    unsafe { index_buf.set_len(0) };
                    unsafe { ledger_diff_buf.set_len(0) };
                }
                let mut write_options = WriteOptions::default();
                write_options.set_sync(false);
                write_options.disable_wal(true);
                storage.inner.write_opt(batch, &write_options)?;
                Ok(())
            }
            async fn remove(milestone_index: &MilestoneIndex, storage: &Storage) -> Result<(), OpError> {
                let db = &storage.inner;
                let ms_index_to_ledger_diff = db.cf_handle(MILESTONE_INDEX_TO_LEDGER_DIFF).unwrap();
                let mut index_buf = Vec::new();
                milestone_index.encode_persistable(&mut index_buf);
                db.delete_cf(&ms_index_to_ledger_diff, index_buf.as_slice())?;
                Ok(())
            }
            async fn find_by_milestone_index(
                milestone_index: &MilestoneIndex,
                storage: &Storage,
            ) -> Result<Option<Self>, OpError> {
                let ms_index_to_ledger_diff = storage.inner.cf_handle(MILESTONE_INDEX_TO_LEDGER_DIFF).unwrap();
                let mut index_buf: Vec<u8> = Vec::new();
                milestone_index.encode_persistable(&mut index_buf);
                if let Some(res) = storage
                    .inner
                    .get_cf(&ms_index_to_ledger_diff, index_buf.as_slice())?
                {
                    let ledger_diff: Self = Self::decode_persistable(res.as_slice());
                    Ok(Some(ledger_diff))
                } else {
                    Ok(None)
                }
            }
        }
    };
}
