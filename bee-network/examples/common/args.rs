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

use bee_network::{Address, Url};

use super::config::Config;

use async_std::task::block_on;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "pingpong", about = "bee-network example")]
pub struct Args {
    #[structopt(long)]
    pub bind: String,

    #[structopt(long)]
    pub peers: Vec<String>,

    #[structopt(long)]
    pub msg: String,
}

impl Args {
    pub fn make_config(&self) -> Config {
        let mut peers = vec![];
        for peer in &self.peers {
            peers.push(block_on(Url::from_url_str(&peer)).unwrap());
        }

        Config {
            host_addr: block_on(Address::from_addr_str(&self.bind.clone()[..])).unwrap(),
            peers,
        }
    }
}
