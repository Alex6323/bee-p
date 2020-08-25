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
            persistable::Persistable,
            storage::{rocksdb::*, Storage},
        };
        use bee_transaction::bundled::BundledTransactionField;
        #[cfg(feature = "rocks_db")]
        impl $object {
            async fn insert(&self, storage: &Storage) -> Result<(), OpError> {
                let db = &storage.inner;
                let milestone_hash_to_index = db.cf_handle(MILESTONE_HASH_TO_INDEX).unwrap();
                db.put_cf(
                    &milestone_hash_to_index,
                    self.hash().encode_as_slice(),
                    self.index().encode_as_slice(),
                )?;
                Ok(())
            }
            async fn remove(hash: &Hash, storage: &Storage) -> Result<(), OpError>
            where
                Hash: Persistable,
            {
                let db = &storage.inner;
                let milestone_hash_to_index = db.cf_handle(MILESTONE_HASH_TO_INDEX).unwrap();
                db.delete_cf(&milestone_hash_to_index, hash.encode_as_slice())?;
                Ok(())
            }
            async fn find_by_hash(hash: &Hash, storage: &Storage) -> Result<Option<Self>, OpError>
            where
                Hash: Persistable,
            {
                let milestone_hash_to_index = storage.inner.cf_handle(MILESTONE_HASH_TO_INDEX).unwrap();
                if let Some(res) = storage
                    .inner
                    .get_cf(&milestone_hash_to_index, hash.encode_as_slice())?
                {
                    Ok(Some(Milestone::new(
                        hash,
                        MilestoneIndex::decode(res.as_slice(), res.len()),
                    )))
                } else {
                    Ok(None)
                }
            }
        }
    };
}
