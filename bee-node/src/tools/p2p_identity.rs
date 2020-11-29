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

use bee_network::{Keypair, PeerId, PublicKey};

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct P2pIdentityTool {}

pub fn exec(_tool: &P2pIdentityTool) {
    let keypair = Keypair::generate();
    let public = keypair.public();

    println!("Your p2p private key:\t{}", hex::encode(keypair.encode()));
    println!("Your p2p public key:\t{}", hex::encode(public.encode()));
    println!(
        "Your p2p PeerID:\t{}",
        PeerId::from_public_key(PublicKey::Ed25519(public))
    );
}
