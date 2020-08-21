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
        use bee_storage::{
            access::OpError,
            storage::{rocksdb::*, Storage},
        };
        use bee_ternary::{T5B1Buf, TritBuf, T5B1};
        use bee_transaction::bundled::BundledTransactionField;
        #[cfg(feature = "rocks_db")]
        impl $object {
            async fn insert(&self, storage: &Storage) -> Result<(), OpError> {
                let db = &storage.inner;
                let milestone_hash_to_index = db.cf_handle(MILESTONE_HASH_TO_INDEX).unwrap();
                let hash_buf = self.hash().encode::<T5B1Buf>().as_i8_slice();
                db.put_cf(
                    &milestone_hash_to_index,
                    cast_slice(hash_buf),
                    self.index().to_le_bytes(),
                )?;
                Ok(())
            }
            async fn remove(hash: &Hash, storage: &Storage) -> Result<(), OpError> {
                let db = &storage.inner;
                let milestone_hash_to_index = db.cf_handle(MILESTONE_HASH_TO_INDEX).unwrap();
                let hash_buf = self.hash().encode::<T5B1Buf>().as_i8_slice();
                db.delete_cf(&milestone_hash_to_index, cast_slice(hash_buf))?;
                Ok(())
            }
            async fn find_by_hash(hash: &Hash, storage: &Storage) -> Result<Option<Self>, OpError> {
                let milestone_hash_to_index = storage.inner.cf_handle(MILESTONE_HASH_TO_INDEX).unwrap();
                if let Some(res) = storage.inner.get_cf(
                    &milestone_hash_to_index,
                    cast_slice(hash.encode::<T5B1Buf>().as_i8_slice()),
                )? {
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
