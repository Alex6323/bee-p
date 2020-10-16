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
use bee_protocol::{tangle::MessageMetadata, MilestoneIndex};
use bee_storage::{access::Delete, persistable::Persistable};

use crate::{access::OpError, storage::*};

#[async_trait::async_trait]
impl Delete<Hash, MessageMetadata> for Storage {
    type Error = OpError;
    async fn delete(&self, hash: &Hash) -> Result<(), Self::Error> {
        let db = &self.inner;
        let hash_to_metadata = db.cf_handle(TRANSACTION_HASH_TO_METADATA).unwrap();
        let mut hash_buf = Vec::new();
        hash.write_to(&mut hash_buf);
        db.delete_cf(&hash_to_metadata, hash_buf.as_slice())?;
        Ok(())
    }
}

// #[async_trait::async_trait]
// impl Delete<MilestoneIndex, LedgerDiff> for Storage {
//     type Error = OpError;
//     async fn delete(&self, milestone_index: &MilestoneIndex) -> Result<(), Self::Error> {
//         let db = &self.inner;
//         let ms_index_to_ledger_diff = db.cf_handle(MILESTONE_INDEX_TO_LEDGER_DIFF).unwrap();
//         let mut index_buf = Vec::new();
//         milestone_index.write_to(&mut index_buf);
//         db.delete_cf(&ms_index_to_ledger_diff, index_buf.as_slice())?;
//         Ok(())
//     }
// }
//
// #[async_trait::async_trait]
// impl Delete<MilestoneIndex, LedgerState> for Storage {
//     type Error = OpError;
//     async fn delete(&self, milestone_index: &MilestoneIndex) -> Result<(), Self::Error> {
//         let db = &self.inner;
//         let ms_index_to_ledger_state = db.cf_handle(MILESTONE_INDEX_TO_LEDGER_STATE).unwrap();
//         let mut index_buf = Vec::new();
//         milestone_index.write_to(&mut index_buf);
//         db.delete_cf(&ms_index_to_ledger_state, index_buf.as_slice())?;
//         Ok(())
//     }
// }

#[async_trait::async_trait]
impl Delete<Hash, MilestoneIndex> for Storage {
    type Error = OpError;
    async fn delete(&self, hash: &Hash) -> Result<(), Self::Error> {
        let db = &self.inner;
        let ms_hash_to_ms_index = db.cf_handle(MILESTONE_HASH_TO_INDEX).unwrap();
        let mut hash_buf = Vec::new();
        hash.write_to(&mut hash_buf);
        db.delete_cf(&ms_hash_to_ms_index, hash_buf.as_slice())?;
        Ok(())
    }
}
