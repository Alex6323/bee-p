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

use bee_crypto::ternary::Hash;
use bee_ledger::diff::LedgerDiff;
use bee_protocol::{tangle::TransactionMetadata, MilestoneIndex};
use bee_storage::{
    access::{Batch, BatchBuilder},
    persistable::Persistable,
};
use bee_transaction::bundled::BundledTransaction;

use crate::{access::OpError, storage::*};

pub struct StorageBatch<'a, K, V> {
    storage: &'a Storage,
    batch: WriteBatch,
    key_buf: Vec<u8>,
    value_buf: Vec<u8>,
    phantom: std::marker::PhantomData<(K, V)>,
}

impl<'a, K, V> Batch<'a, K, V> for Storage {
    type BatchBuilder = StorageBatch<'a, K, V>;
    fn create_batch(&'a self) -> Self::BatchBuilder {
        StorageBatch {
            storage: self,
            batch: WriteBatch::default(),
            key_buf: Vec::new(),
            value_buf: Vec::new(),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'a> BatchBuilder<'a, Storage, Hash, TransactionMetadata> for StorageBatch<'a, Hash, TransactionMetadata> {
    type Error = OpError;
    fn try_insert(
        mut self,
        hash: Hash,
        transaction_metadata: TransactionMetadata,
    ) -> Result<Self, (Self, Self::Error)> {
        let hash_to_metadata = self.storage.inner.cf_handle(TRANSACTION_HASH_TO_METADATA).unwrap();
        self.key_buf.clear();
        self.value_buf.clear();
        hash.encode_persistable::<Self>(&mut self.key_buf);
        transaction_metadata.encode_persistable::<Self>(&mut self.value_buf);
        self.batch
            .put_cf(&hash_to_metadata, self.key_buf.as_slice(), self.value_buf.as_slice());
        Ok(self)
    }
    fn try_delete(mut self, hash: Hash) -> Result<Self, (Self, Self::Error)> {
        let hash_to_metadata = self.storage.inner.cf_handle(TRANSACTION_HASH_TO_METADATA).unwrap();
        self.key_buf.clear();
        hash.encode_persistable::<Self>(&mut self.key_buf);
        self.batch.delete_cf(&hash_to_metadata, self.key_buf.as_slice());
        Ok(self)
    }
    fn apply(self, durability: bool) -> Result<(), Self::Error> {
        let mut write_options = WriteOptions::default();
        write_options.set_sync(false);
        write_options.disable_wal(!durability);
        self.storage.inner.write_opt(self.batch, &write_options)?;
        Ok(())
    }
}

impl<'a> BatchBuilder<'a, Storage, MilestoneIndex, LedgerDiff> for StorageBatch<'a, MilestoneIndex, LedgerDiff> {
    type Error = OpError;
    fn try_insert(mut self, ms_index: MilestoneIndex, ledger_diff: LedgerDiff) -> Result<Self, (Self, Self::Error)> {
        let ms_index_to_ledger_diff = self.storage.inner.cf_handle(MILESTONE_INDEX_TO_LEDGER_DIFF).unwrap();
        self.key_buf.clear();
        self.value_buf.clear();
        ms_index.encode_persistable::<Self>(&mut self.key_buf);
        ledger_diff.encode_persistable::<Self>(&mut self.value_buf);
        self.batch.put_cf(
            &ms_index_to_ledger_diff,
            self.key_buf.as_slice(),
            self.value_buf.as_slice(),
        );
        Ok(self)
    }

    fn try_delete(mut self, ms_index: MilestoneIndex) -> Result<Self, (Self, Self::Error)> {
        let ms_index_to_ledger_diff = self.storage.inner.cf_handle(MILESTONE_INDEX_TO_LEDGER_DIFF).unwrap();
        self.key_buf.clear();
        ms_index.encode_persistable::<Self>(&mut self.key_buf);
        self.batch.delete_cf(&ms_index_to_ledger_diff, self.key_buf.as_slice());
        Ok(self)
    }

    fn apply(self, durability: bool) -> Result<(), Self::Error> {
        let mut write_options = WriteOptions::default();
        write_options.set_sync(false);
        write_options.disable_wal(!durability);
        self.storage.inner.write_opt(self.batch, &write_options)?;
        Ok(())
    }
}

impl<'a> BatchBuilder<'a, Storage, Hash, BundledTransaction> for StorageBatch<'a, Hash, BundledTransaction> {
    type Error = OpError;
    fn try_insert(mut self, hash: Hash, bundled_transaction: BundledTransaction) -> Result<Self, (Self, Self::Error)> {
        let hash_to_tx = self.storage.inner.cf_handle(TRANSACTION_HASH_TO_TRANSACTION).unwrap();
        self.key_buf.clear();
        self.value_buf.clear();
        hash.encode_persistable::<Self>(&mut self.key_buf);
        bundled_transaction.encode_persistable::<Self>(&mut self.value_buf);
        self.batch
            .put_cf(&hash_to_tx, self.key_buf.as_slice(), self.value_buf.as_slice());
        Ok(self)
    }

    fn try_delete(mut self, hash: Hash) -> Result<Self, (Self, Self::Error)> {
        let hash_to_tx = self.storage.inner.cf_handle(TRANSACTION_HASH_TO_TRANSACTION).unwrap();
        self.key_buf.clear();
        hash.encode_persistable::<Self>(&mut self.key_buf);
        self.batch.delete_cf(&hash_to_tx, self.key_buf.as_slice());
        Ok(self)
    }

    fn apply(self, durability: bool) -> Result<(), Self::Error> {
        let mut write_options = WriteOptions::default();
        write_options.set_sync(false);
        write_options.disable_wal(!durability);
        self.storage.inner.write_opt(self.batch, &write_options)?;
        Ok(())
    }
}

impl<'a> BatchBuilder<'a, Storage, Hash, MilestoneIndex> for StorageBatch<'a, Hash, MilestoneIndex> {
    type Error = OpError;
    fn try_insert(mut self, hash: Hash, milestone_index: MilestoneIndex) -> Result<Self, (Self, Self::Error)> {
        let ms_hash_to_ms_index = self.storage.inner.cf_handle(MILESTONE_HASH_TO_INDEX).unwrap();
        self.key_buf.clear();
        self.value_buf.clear();
        hash.encode_persistable::<Self>(&mut self.key_buf);
        milestone_index.encode_persistable::<Self>(&mut self.value_buf);
        self.batch
            .put_cf(&ms_hash_to_ms_index, self.key_buf.as_slice(), self.value_buf.as_slice());
        Ok(self)
    }

    fn try_delete(mut self, hash: Hash) -> Result<Self, (Self, Self::Error)> {
        let ms_hash_to_ms_index = self.storage.inner.cf_handle(MILESTONE_HASH_TO_INDEX).unwrap();
        self.key_buf.clear();
        hash.encode_persistable::<Self>(&mut self.key_buf);
        self.batch.delete_cf(&ms_hash_to_ms_index, self.key_buf.as_slice());
        Ok(self)
    }

    fn apply(self, durability: bool) -> Result<(), Self::Error> {
        let mut write_options = WriteOptions::default();
        write_options.set_sync(false);
        write_options.disable_wal(!durability);
        self.storage.inner.write_opt(self.batch, &write_options)?;
        Ok(())
    }
}
