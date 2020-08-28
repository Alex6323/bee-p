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
pub trait TransactionMetadataOps<H, S, E> {
    async fn insert(&self, hash: &H, storage: &S) -> Result<(), E>
    where
        Self: Persistable + Sized,
        H: Persistable,
        S: Backend;
    async fn insert_batch(metadatas: &HashMap<H, Self>, storage: &S) -> Result<(), E>
    where
        Self: Persistable + Sized,
        H: Persistable,
        S: Backend;
    async fn remove(hash: &H, storage: &S) -> Result<(), E>
    where
        Self: Persistable + Sized,
        H: Persistable,
        S: Backend;
    async fn find_by_hash(hash: &H, storage: &S) -> Result<Option<Self>, E>
    where
        Self: Persistable + Sized,
        H: Persistable,
        S: Backend;
}

#[macro_export]
#[cfg(feature = "rocks_db")]
macro_rules! impl_transaction_metadata_ops {
    ($object:ty) => {
        use bee_storage::{
            access::{OpError, TransactionMetadataOps},
            storage::{rocksdb::*, Backend, Storage},
        };
        use std::collections::HashMap;
        #[async_trait::async_trait]
        impl TransactionOps<Hash, Storage, OpError> for $object {
            async fn insert(&self, hash: &Hash, storage: &Storage) -> Result<(), OpError> {
                // get column family handle to hash_to_metadata table in order presist the transaction_metadata;
                let hash_to_metadata = storage.inner.cf_handle(TRANSACTION_HASH_TO_METADATA).unwrap();
                let mut hash_buf = Vec::new();
                hash.encode_persistable(&mut hash_buf);
                let mut metadata_buf = Vec::new();
                self.encode_persistable(&mut metadata_buf);
                storage
                    .inner
                    .put_cf(&hash_to_metadata, hash_buf.as_slice(), metadata_buf.as_slice())?;
                Ok(())
            }
            async fn insert_batch(metadatas: &HashMap<Hash, Self>, storage: &Storage) -> Result<(), OpError> {
                let mut batch = WriteBatch::default();
                let hash_to_metadata = storage.inner.cf_handle(TRANSACTION_HASH_TO_METADATA).unwrap();
                // reusable buffers
                let mut hash_buf: Vec<u8> = Vec::new();
                let mut metadata_buf: Vec<u8> = Vec::new();
                for (hash, metadata) in metadatas {
                    hash.encode_persistable(&mut hash_buf);
                    metadata.encode_persistable(&mut metadata_buf);
                    batch.put_cf(&hash_to_metadata, hash_buf.as_slice(), metadata_buf.as_slice());
                    // note: for optimization reason we used buf.set_len = 0 instead of clear()
                    unsafe { hash_buf.set_len(0) };
                    unsafe { metadata_buf.set_len(0) };
                }
                let mut write_options = WriteOptions::default();
                write_options.set_sync(false);
                write_options.disable_wal(true);
                storage.inner.write_opt(batch, &write_options)?;
                Ok(())
            }
            async fn remove(hash: &Hash, storage: &Storage) -> Result<(), OpError> {
                let db = &storage.inner;
                let hash_to_metadata = db.cf_handle(TRANSACTION_HASH_TO_METADATA).unwrap();
                let mut hash_buf = Vec::new();
                hash.encode_persistable(&mut hash_buf);
                db.delete_cf(&hash_to_metadata, hash_buf.as_slice())?;
                Ok(())
            }
            async fn find_by_hash(hash: &Hash, storage: &Storage) -> Result<Option<Self>, OpError> {
                let hash_to_metadata = storage.inner.cf_handle(TRANSACTION_HASH_TO_METADATA).unwrap();
                let mut hash_buf: Vec<u8> = Vec::new();
                hash.encode_persistable(&mut hash_buf);
                if let Some(res) = storage.inner.get_cf(&hash_to_metadata, hash_buf.as_slice())? {
                    let transaction_metadata: Self = Self::decode_persistable(res.as_slice(), res.len());
                    Ok(Some(transaction_metadata))
                } else {
                    Ok(None)
                }
            }
        }
    };
}
