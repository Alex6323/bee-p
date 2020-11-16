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

use super::config::Config;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "pingpong", about = "bee-network example")]
pub struct Args {
    #[structopt(short = "b", long = "bind")]
    bind_address: String,

    #[structopt(short = "p", long = "peers")]
    peer_addresses: Vec<String>,

    #[structopt(short = "m", long = "msg")]
    message: String,
}

impl Args {
    pub fn into_config(self) -> Config {
        let Args {
            bind_address,
            mut peer_addresses,
            message,
        } = self;

        let mut config = Config::build().with_bind_address(bind_address).with_message(message);

        for peer_address in peer_addresses.drain(..) {
            config = config.with_peer_address(peer_address);
        }

        config.finish()
    }
}
