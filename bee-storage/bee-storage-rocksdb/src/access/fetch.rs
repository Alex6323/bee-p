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
// use bee_ledger::{diff::LedgerDiff, state::LedgerState};
use bee_protocol::{tangle::TransactionMetadata, MilestoneIndex};
use bee_storage::{access::Fetch, persistable::Persistable};

use crate::{access::OpError, storage::*};

#[async_trait::async_trait]
impl Fetch<Hash, TransactionMetadata> for Storage {
    type Error = OpError;
    async fn fetch(&self, hash: &Hash) -> Result<Option<TransactionMetadata>, OpError>
    where
        Self: Sized,
    {
        let hash_to_metadata = self.inner.cf_handle(TRANSACTION_HASH_TO_METADATA).unwrap();
        let mut hash_buf: Vec<u8> = Vec::new();
        hash.write_to(&mut hash_buf);
        if let Some(res) = self.inner.get_cf(&hash_to_metadata, hash_buf.as_slice())? {
            let transaction_metadata: TransactionMetadata = TransactionMetadata::read_from(res.as_slice());
            Ok(Some(transaction_metadata))
        } else {
            Ok(None)
        }
    }
}

// #[async_trait::async_trait]
// impl Fetch<MilestoneIndex, LedgerDiff> for Storage {
//     type Error = OpError;
//     async fn fetch(&self, milestone_index: &MilestoneIndex) -> Result<Option<LedgerDiff>, OpError>
//     where
//         Self: Sized,
//     {
//         let ms_index_to_ledger_diff = self.inner.cf_handle(MILESTONE_INDEX_TO_LEDGER_DIFF).unwrap();
//         let mut index_buf: Vec<u8> = Vec::new();
//         milestone_index.write_to(&mut index_buf);
//         if let Some(res) = self
//             .inner
//             .get_cf(&ms_index_to_ledger_diff, index_buf.as_slice())
//             .unwrap()
//         {
//             let ledger_diff: LedgerDiff = LedgerDiff::read_from(res.as_slice());
//             Ok(Some(ledger_diff))
//         } else {
//             Ok(None)
//         }
//     }
// }
//
// #[async_trait::async_trait]
// impl Fetch<MilestoneIndex, LedgerState> for Storage {
//     type Error = OpError;
//     async fn fetch(&self, milestone_index: &MilestoneIndex) -> Result<Option<LedgerState>, OpError>
//     where
//         Self: Sized,
//     {
//         let ms_index_to_ledger_state = self.inner.cf_handle(MILESTONE_INDEX_TO_LEDGER_STATE).unwrap();
//         let mut index_buf: Vec<u8> = Vec::new();
//         milestone_index.write_to(&mut index_buf);
//         if let Some(res) = self
//             .inner
//             .get_cf(&ms_index_to_ledger_state, index_buf.as_slice())
//             .unwrap()
//         {
//             let ledger_state: LedgerState = LedgerState::read_from(res.as_slice());
//             Ok(Some(ledger_state))
//         } else {
//             Ok(None)
//         }
//     }
// }

#[async_trait::async_trait]
impl Fetch<Hash, MilestoneIndex> for Storage {
    type Error = OpError;
    async fn fetch(&self, hash: &Hash) -> Result<Option<MilestoneIndex>, Self::Error>
    where
        Self: Sized,
    {
        let ms_hash_to_ms_index = self.inner.cf_handle(MILESTONE_HASH_TO_INDEX).unwrap();
        let mut hash_buf: Vec<u8> = Vec::new();
        hash.write_to(&mut hash_buf);
        if let Some(res) = self.inner.get_cf(&ms_hash_to_ms_index, hash_buf.as_slice())? {
            let ms_index: MilestoneIndex = MilestoneIndex::read_from(res.as_slice());
            Ok(Some(ms_index))
        } else {
            Ok(None)
        }
    }
}
