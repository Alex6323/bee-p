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
macro_rules! impl_transaction_ops {
    ($object:ty) => {
        use bee_storage::{
            access::OpError,
            storage::{rocksdb::*, Storage},
        };
        use bee_ternary::{T5B1Buf, TritBuf, T5B1};
        use std::collections::HashMap;
        #[cfg(feature = "rocks_db")]
        impl $object {
            async fn insert(&self, hash: &Hash, storage: &Storage) -> Result<(), OpError>
            where
                Self: Persistable,
                Hash: Persistable,
            {
                // get column family handle to hash_to_tx table in order presist the transaction;
                let hash_to_tx = storage.inner.cf_handle(TRANSACTION_HASH_TO_TRANSACTION).unwrap();
                let mut hash_buf: Vec<u8> = Vec::new();
                hash.encode(&mut hash_buf);
                let mut tx_buf: Vec<u8> = Vec::new();
                self.encode(&mut tx_buf);
                storage
                    .inner
                    .put_cf(&hash_to_tx, hash_buf.as_slice(), tx_buf.as_slice())?;
                Ok(())
            }
            async fn insert_batch(transactions: &HashMap<Hash, Self>, storage: &Storage) -> Result<(), OpError>
            where
                Hash: Persistable,
                Self: Persistable,
            {
                let mut batch = rocksdb::WriteBatch::default();
                let hash_to_tx = storage.inner.cf_handle(TRANSACTION_HASH_TO_TRANSACTION).unwrap();
                // reusable buffers
                let mut hash_buf: Vec<u8> = Vec::new();
                let mut tx_buf: Vec<u8> = Vec::new();
                for (hash, tx) in transactions {
                    batch.put_cf(
                        &hash_to_tx,
                        hash.encode(&mut hash_buf).as_slice(),
                        tx.encode(&mut tx_buf).as_slice(),
                    );
                    // note: for optimization reason we used buf.set_len = 0 instead of clear()
                    unsafe { hash_buf.set_len(0) };
                    unsafe { tx_buf.set_len(0) };
                }
                let mut write_options = rocksdb::WriteOptions::default();
                write_options.set_sync(false);
                write_options.disable_wal(true);
                storage.inner.write_opt(batch, &write_options)?;
                Ok(())
            }
            async fn remove(hash: &Hash, storage: &Storage) -> Result<(), OpError> {
                let db = &storage.inner;
                let hash_to_tx = db.cf_handle(TRANSACTION_HASH_TO_TRANSACTION).unwrap();
                let mut hash_buf: Vec<u8> = Vec::new();
                hash.encode(&mut hash_buf);
                db.delete_cf(&hash_to_tx, hash_buf.as_slice())?;
                Ok(())
            }
            async fn find_by_hash(hash: &Hash, storage: &Storage) -> Result<Option<Self>, OpError>
            where
                Self: Persistable,
                Hash: Persistable,
            {
                let hash_to_tx = storage.inner.cf_handle(TRANSACTION_HASH_TO_TRANSACTION).unwrap();
                let mut hash_buf: Vec<u8> = Vec::new();
                hash.encode(&mut hash_buf);
                if let Some(res) = storage.inner.get_cf(&hash_to_tx, hash_buf.as_slice())? {
                    let transaction: Self = Self::decode(res.as_slice(), res.len());
                    Ok(Some(transaction))
                } else {
                    Ok(None)
                }
            }
        }
    };
}
