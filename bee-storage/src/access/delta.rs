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
macro_rules! impl_delta_ops {
    ($object:ty) => {
        use bee_storage::{
            access::OpError,
            storage::{rocksdb::*, Storage},
        };
        use bee_ternary::{T5B1Buf, TritBuf, T5B1};
        use std::collections::HashSet;
        #[cfg(feature = "rocks_db")]
        impl $object {
            async fn insert(&self, milestone_index: &MilestoneIndex, storage: &Storage) -> Result<(), OpError> {
                let db = &storage.inner;
                let milestone_index_to_delta = db.cf_handle(MILESTONE_INDEX_TO_DELTA).unwrap();
                let delta_buf: Self = bincode::serialize(&self).unwrap();
                db.put_cf(
                    &milestone_index_to_delta,
                    milestone_index.to_le_bytes(),
                    cast_slice(delta_buf),
                )?;
                Ok(())
            }
            async fn remove(milestone_index: &MilestoneIndex, storage: &Storage) -> Result<(), OpError> {
                let db = &storage.inner;
                let milestone_index_to_delta = db.cf_handle(MILESTONE_INDEX_TO_DELTA).unwrap();
                db.delete_cf(&milestone_index_to_delta, milestone_index.to_le_bytes())?;
                Ok(())
            }
            async fn find_by_milestone_index(
                milestone_index: &MilestoneIndex,
                storage: &Storage,
            ) -> Result<Option<Self>, OpError> {
                let milestone_index_to_delta = storage.inner.cf_handle(MILESTONE_INDEX_TO_DELTA).unwrap();
                if let Some(res) = storage
                    .inner
                    .get_cf(&milestone_index_to_delta, milestone_index.to_le_bytes())?
                {
                    let delta: Self = bincode::deserialize(&res[..]).unwrap();
                    Ok(Some(delta))
                } else {
                    Ok(None)
                }
            }
        }
    };
}
