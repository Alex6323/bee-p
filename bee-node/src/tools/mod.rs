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

mod ed25519;
mod p2p_identity;
mod snapshot_info;

use structopt::StructOpt;

#[non_exhaustive]
#[derive(Debug, StructOpt)]
pub enum Tool {
    /// Generates a set of Ed25519 public and private keys.
    Ed25519(ed25519::Ed25519Tool),
    /// Generates a p2p identity.
    P2pIdentity(p2p_identity::P2pIdentityTool),
    /// Reads information from a snapshot file.
    SnapshotInfo(snapshot_info::SnapshotInfo),
}

pub fn exec(tool: &Tool) {
    match tool {
        Tool::Ed25519(tool) => ed25519::exec(tool),
        Tool::P2pIdentity(tool) => p2p_identity::exec(tool),
        Tool::SnapshotInfo(tool) => snapshot_info::exec(tool),
    }
}
