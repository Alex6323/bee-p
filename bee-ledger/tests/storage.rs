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

use bee_ledger::diff::*;

use bee_protocol::MilestoneIndex;

#[async_std::test]
async fn persist_ledger_diff() {
    // start storage
    let storage: Storage = Storage::start("../bee-storage/config.toml".to_string()).await.unwrap();
    // create empty ledger_diff
    let ledger_diff: LedgerDiff = LedgerDiff::new();
    // milestone_index
    let ms = MilestoneIndex(0);
    // persist it
    assert!(ledger_diff.insert(&ms, &storage).await.is_ok());
    // find it
    if let Ok(same_ledger_diff) = LedgerDiff::find_by_milestone_index(&ms, &storage).await {
        assert!(same_ledger_diff.is_some());
    } else {
        panic!("persist_ledger_diff test")
    };
    // shutdown storage
    assert!(storage.shutdown().await.is_ok())
}
