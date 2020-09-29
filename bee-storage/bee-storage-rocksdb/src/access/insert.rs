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
use bee_ledger::{diff::LedgerDiff, state::LedgerState};
use bee_protocol::{tangle::TransactionMetadata, MilestoneIndex};
use bee_storage::{access::Insert, persistable::Persistable};
use bee_transaction::bundled::BundledTransaction;

use crate::{access::OpError, storage::*};

#[async_trait::async_trait]
impl Insert<Hash, TransactionMetadata> for Storage {
    type Error = OpError;
    async fn insert(&self, hash: &Hash, tx_metadata: &TransactionMetadata) -> Result<(), Self::Error> {
        let hash_to_metadata = self.inner.cf_handle(TRANSACTION_HASH_TO_METADATA).unwrap();
        let mut hash_buf = Vec::new();
        hash.write_to(&mut hash_buf);
        let mut metadata_buf = Vec::new();
        tx_metadata.write_to(&mut metadata_buf);
        self.inner
            .put_cf(&hash_to_metadata, hash_buf.as_slice(), metadata_buf.as_slice())?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<MilestoneIndex, LedgerDiff> for Storage {
    type Error = OpError;
    async fn insert(&self, milestone_index: &MilestoneIndex, ledger_diff: &LedgerDiff) -> Result<(), Self::Error> {
        let ms_index_to_ledger_diff = self.inner.cf_handle(MILESTONE_INDEX_TO_LEDGER_DIFF).unwrap();
        let mut index_buf = Vec::new();
        milestone_index.write_to(&mut index_buf);
        let mut ledger_diff_buf = Vec::new();
        ledger_diff.write_to(&mut ledger_diff_buf);
        self.inner.put_cf(
            &ms_index_to_ledger_diff,
            index_buf.as_slice(),
            ledger_diff_buf.as_slice(),
        )?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<MilestoneIndex, LedgerState> for Storage {
    type Error = OpError;
    async fn insert(&self, milestone_index: &MilestoneIndex, ledger_state: &LedgerState) -> Result<(), Self::Error> {
        let ms_index_to_ledger_state = self.inner.cf_handle(MILESTONE_INDEX_TO_LEDGER_STATE).unwrap();
        let mut index_buf = Vec::new();
        milestone_index.write_to(&mut index_buf);
        let mut ledger_state_buf = Vec::new();
        ledger_state.write_to(&mut ledger_state_buf);
        self.inner.put_cf(
            &ms_index_to_ledger_state,
            index_buf.as_slice(),
            ledger_state_buf.as_slice(),
        )?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<Hash, BundledTransaction> for Storage {
    type Error = OpError;
    async fn insert(&self, hash: &Hash, bundle_transaction: &BundledTransaction) -> Result<(), Self::Error> {
        let hash_to_tx = self.inner.cf_handle(TRANSACTION_HASH_TO_TRANSACTION).unwrap();
        let mut hash_buf = Vec::new();
        hash.write_to(&mut hash_buf);
        let mut tx_buf = Vec::new();
        bundle_transaction.write_to(&mut tx_buf);
        self.inner.put_cf(&hash_to_tx, hash_buf.as_slice(), tx_buf.as_slice())?;
        Ok(())
    }
}
#[async_trait::async_trait]
impl Insert<Hash, MilestoneIndex> for Storage {
    type Error = OpError;
    async fn insert(&self, hash: &Hash, milestone_index: &MilestoneIndex) -> Result<(), Self::Error> {
        let ms_hash_to_ms_index = self.inner.cf_handle(MILESTONE_HASH_TO_INDEX).unwrap();
        let mut hash_buf = Vec::new();
        hash.write_to(&mut hash_buf);
        let mut index_buf = Vec::new();
        milestone_index.write_to(&mut index_buf);
        self.inner
            .put_cf(&ms_hash_to_ms_index, hash_buf.as_slice(), index_buf.as_slice())?;
        Ok(())
    }
}
