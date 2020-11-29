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

use bee_snapshot::snapshot::Snapshot;

use chrono::{offset::TimeZone, Utc};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct SnapshotInfo {
    path: String,
}

pub fn exec(tool: &SnapshotInfo) {
    let snapshot = Snapshot::from_file(&tool.path).unwrap();

    println!("Type:\t\t\t{:?}", snapshot.header().kind());
    println!(
        "Timestamp:\t\t{} ({})",
        snapshot.header().timestamp(),
        Utc.timestamp(snapshot.header().timestamp() as i64, 0).to_rfc2822()
    );
    println!("Network ID:\t\t{}", snapshot.header().network_id());
    println!("SEP index:\t\t{}", snapshot.header().sep_index());
    println!("Ledger index:\t\t{}", snapshot.header().ledger_index());
    println!("SEP count:\t\t{}", snapshot.solid_entry_points().len());
    println!("Outputs count:\t\t{}", snapshot.outputs_len());
    println!("Milestone diffs count:\t{}", snapshot.milestone_diffs_len());
}
