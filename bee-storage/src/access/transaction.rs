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
            async fn insert(&self, hash: &Hash, storage: &Storage) -> Result<(), OpError> {
                // get column family handle to hash_to_tx table in order presist the transaction;
                let hash_to_tx = storage.inner.cf_handle(TRANSACTION_HASH_TO_TRANSACTION).unwrap();
                let mut tx_trit_buf = TritBuf::<T1B1Buf>::zeros(Self::trit_len());
                self.into_trits_allocated(tx_trit_buf.as_slice_mut());
                let hash_buf = hash.encode::<T5B1Buf>();
                storage.inner.put_cf(
                    &hash_to_tx,
                    cast_slice(hash_buf.as_i8_slice()),
                    cast_slice(tx_trit_buf.encode::<T5B1Buf>().as_i8_slice()),
                )?;
                Ok(())
            }
            async fn insert_batch(transactions: &HashMap<Hash, Transaction>, storage: &Storage) -> Result<(), OpError> {
                let mut batch = rocksdb::WriteBatch::default();
                let hash_to_tx = storage.inner.cf_handle(TRANSACTION_HASH_TO_TRANSACTION).unwrap();
                let mut tx_trit_buf = TritBuf::<T1B1Buf>::zeros(Transaction::trit_len());
                for (hash, tx) in transactions {
                    tx.into_trits_allocated(tx_trit_buf.as_slice_mut());
                    let hash_buf = hash.encode::<T5B1Buf>();
                    batch.put_cf(
                        &hash_to_tx,
                        cast_slice(hash_buf.as_i8_slice()),
                        cast_slice(tx_trit_buf.encode::<T5B1Buf>().as_i8_slice()),
                    );
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
                let hash_buf = self.hash().encode::<T5B1Buf>().as_i8_slice();
                db.delete_cf(&hash_to_tx, cast_slice(hash_buf))?;
                Ok(())
            }
            async fn find_by_hash(hash: &Hash, storage: &Storage) -> Result<Option<Self>, OpError> {
                let hash_to_tx = storage.inner.cf_handle(TRANSACTION_HASH_TO_TRANSACTION).unwrap();
                if let Some(res) = storage
                    .inner
                    .get_cf(&hash_to_tx, cast_slice(hash.encode::<T5B1Buf>().as_i8_slice()))?
                {
                    let trits = unsafe { Trits::<T5B1>::from_raw_unchecked(&cast_slice(&res), Self::trit_len()) }
                        .encode::<T1B1Buf>();
                    let transaction = Self::from_trits(&trits).unwrap();
                    Ok(Some(transaction))
                } else {
                    Ok(None)
                }
            }
        }
    };
}
