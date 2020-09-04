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
use crate::storage::rocksdb::{WriteBatch, WriteOptions, TRANSACTION_HASH_TO_TRANSACTION};
use crate::{persistable::Persistable, storage::Storage};
use std::collections::HashMap;

#[async_trait::async_trait]
#[cfg(feature = "rocks_db")]
pub trait TransactionOps<H: std::marker::Sync> {
    async fn insert(&self, hash: &H, storage: &Storage) -> Result<(), OpError>
    where
        Self: Persistable + Sized,
        H: Persistable,
    {
        // get column family handle to hash_to_tx table in order presist the transaction;
        let hash_to_tx = storage.inner.cf_handle(TRANSACTION_HASH_TO_TRANSACTION).unwrap();
        let mut hash_buf = Vec::new();
        hash.encode_persistable(&mut hash_buf);
        let mut tx_buf = Vec::new();
        self.encode_persistable(&mut tx_buf);
        storage
            .inner
            .put_cf(&hash_to_tx, hash_buf.as_slice(), tx_buf.as_slice())?;
        Ok(())
    }
    async fn insert_batch(transactions: &HashMap<H, Self>, storage: &Storage) -> Result<(), OpError>
    where
        Self: Persistable + Sized + Sync,
        H: Persistable,
    {
        let mut batch = WriteBatch::default();
        let hash_to_tx = storage.inner.cf_handle(TRANSACTION_HASH_TO_TRANSACTION).unwrap();
        // reusable buffers
        let mut hash_buf: Vec<u8> = Vec::new();
        let mut tx_buf: Vec<u8> = Vec::new();
        for (hash, tx) in transactions {
            hash.encode_persistable(&mut hash_buf);
            tx.encode_persistable(&mut tx_buf);
            batch.put_cf(&hash_to_tx, hash_buf.as_slice(), tx_buf.as_slice());
            hash_buf.clear();
            tx_buf.clear();
        }
        let mut write_options = WriteOptions::default();
        write_options.set_sync(false);
        write_options.disable_wal(true);
        storage.inner.write_opt(batch, &write_options)?;
        Ok(())
    }
    async fn remove(hash: &H, storage: &Storage) -> Result<(), OpError>
    where
        Self: Persistable + Sized,
        H: Persistable,
    {
        let db = &storage.inner;
        let hash_to_tx = db.cf_handle(TRANSACTION_HASH_TO_TRANSACTION).unwrap();
        let mut hash_buf = Vec::new();
        hash.encode_persistable(&mut hash_buf);
        db.delete_cf(&hash_to_tx, hash_buf.as_slice())?;
        Ok(())
    }
    async fn find_by_hash(hash: &H, storage: &Storage) -> Result<Option<Self>, OpError>
    where
        Self: Persistable + Sized,
        H: Persistable,
    {
        let hash_to_tx = storage.inner.cf_handle(TRANSACTION_HASH_TO_TRANSACTION).unwrap();
        let mut hash_buf: Vec<u8> = Vec::new();
        hash.encode_persistable(&mut hash_buf);
        if let Some(res) = storage.inner.get_cf(&hash_to_tx, hash_buf.as_slice())? {
            let transaction: Self = Self::decode_persistable(res.as_slice());
            Ok(Some(transaction))
        } else {
            Ok(None)
        }
    }
}
