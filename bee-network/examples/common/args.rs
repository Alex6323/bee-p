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

use std::net::SocketAddr;

#[derive(Debug, StructOpt)]
#[structopt(name = "pingpong", about = "bee-network example")]
pub struct Args {
    #[structopt(short = "b", long = "bind")]
    binding_address: String,

    #[structopt(short = "p", long = "peers")]
    peers: Vec<String>,

    #[structopt(short = "m", long = "msg")]
    message: String,
}

impl Args {
    pub fn config(self) -> Config {
        let Args {
            binding_address,
            peers,
            message,
        } = self;

        Config {
            binding_address: binding_address
                .parse::<SocketAddr>()
                .expect("error parsing binding address"),
            peers,
            message,
        }
    }
}
