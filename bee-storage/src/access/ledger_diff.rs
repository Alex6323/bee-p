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
macro_rules! impl_ledger_diff_ops {
    ($object:ty) => {
        use bee_storage::{
            access::OpError,
            persistable::Persistable,
            storage::{rocksdb::*, Storage},
        };
        #[cfg(feature = "rocks_db")]
        impl $object {
            async fn insert(&self, milestone_index: &MilestoneIndex, storage: &Storage) -> Result<(), OpError>
            where
                Self: Persistable,
            {
                let db = &storage.inner;
                let milestone_index_to_ledger_diff = db.cf_handle(MILESTONE_INDEX_TO_LEDGER_DIFF).unwrap();
                let mut index_buf: Vec<u8> = Vec::new();
                milestone_index.encode(&mut index_buf);
                let mut ledger_diff_buffer: Vec<u8> = Vec::new();
                self.encode(&mut ledger_diff_buffer);
                db.put_cf(
                    &milestone_index_to_ledger_diff,
                    index_buf.as_slice(),
                    ledger_diff_buffer.as_slice(),
                )?;
                Ok(())
            }
            async fn remove(milestone_index: &MilestoneIndex, storage: &Storage) -> Result<(), OpError>
            where
                Self: Persistable,
            {
                let db = &storage.inner;
                let milestone_index_to_ledger_diff = db.cf_handle(MILESTONE_INDEX_TO_LEDGER_DIFF).unwrap();
                let mut index_buf: Vec<u8> = Vec::new();
                milestone_index.encode(&mut index_buf);
                db.delete_cf(&milestone_index_to_ledger_diff, index_buf.as_slice())?;
                Ok(())
            }
            async fn find_by_milestone_index(
                milestone_index: &MilestoneIndex,
                storage: &Storage,
            ) -> Result<Option<Self>, OpError>
            where
                Self: Persistable,
            {
                let milestone_index_to_ledger_diff = storage.inner.cf_handle(MILESTONE_INDEX_TO_LEDGER_DIFF).unwrap();
                let mut index_buf: Vec<u8> = Vec::new();
                milestone_index.encode(&mut index_buf);
                if let Some(res) = storage
                    .inner
                    .get_cf(&milestone_index_to_ledger_diff, index_buf.as_slice())?
                {
                    let ledger_diff: Self = Self::decode(res.as_slice(), res.len());
                    Ok(Some(ledger_diff))
                } else {
                    Ok(None)
                }
            }
        }
    };
}
